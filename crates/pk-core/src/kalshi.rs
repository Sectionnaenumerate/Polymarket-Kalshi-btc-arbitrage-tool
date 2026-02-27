use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{debug, instrument};

use crate::{
    error::PkError,
    types::{KalshiStatus, MarketSide, PriceQuote},
};

const DEFAULT_BASE: &str = "https://api.elections.kalshi.com/trade-api/v2";

pub struct KalshiClient {
    http: Client,
    base: String,
    /// Optional bearer token for authenticated endpoints
    token: Option<String>,
}

// ─── Raw API response shapes ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct KalshiMarketResp {
    market: KalshiMarketData,
}

#[derive(Debug, Deserialize)]
struct KalshiMarketData {
    ticker: String,
    status: String,
    yes_bid: Option<f64>,
    yes_ask: Option<f64>,
    no_bid: Option<f64>,
    no_ask: Option<f64>,
    volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct KalshiOrderbookResp {
    orderbook: KalshiOrderbook,
}

#[derive(Debug, Deserialize)]
struct KalshiOrderbook {
    yes: Vec<[f64; 2]>, // [[price_cents, quantity], ...]
    no: Vec<[f64; 2]>,
}

// ─── Client ──────────────────────────────────────────────────────────────────

impl KalshiClient {
    pub fn new(base: Option<String>, token: Option<String>) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            base: base.unwrap_or_else(|| DEFAULT_BASE.to_string()),
            token,
        }
    }

    /// Fetch YES price (mid of best bid/ask) for a Kalshi BTC 15-min market.
    #[instrument(skip(self))]
    pub async fn get_btc_price(&self, ticker: &str) -> Result<(PriceQuote, KalshiStatus), PkError> {
        let url = format!("{}/markets/{}", self.base, ticker);
        debug!("GET {url}");

        let mut req = self.http.get(&url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let resp: KalshiMarketResp = req.send().await?.json().await?;
        let data = resp.market;

        let status = match data.status.as_str() {
            "open"     => KalshiStatus::Open,
            "closed"   => KalshiStatus::Closed,
            "settled"  => KalshiStatus::Settled,
            _          => KalshiStatus::Unknown,
        };

        // Use mid of bid/ask for best estimate; fall back to bid if ask missing
        let price_cents = match (data.yes_bid, data.yes_ask) {
            (Some(bid), Some(ask)) => Decimal::from_str(&format!("{:.2}", (bid + ask) / 2.0))
                .unwrap_or(Decimal::ZERO),
            (Some(bid), None) => Decimal::from_str(&format!("{:.2}", bid)).unwrap_or(Decimal::ZERO),
            _ => Decimal::ZERO,
        };

        let liquidity = Decimal::from_str(&format!("{:.2}", data.volume.unwrap_or(0.0)))
            .unwrap_or(Decimal::ZERO);

        let quote = PriceQuote {
            exchange: "kalshi".to_string(),
            side: MarketSide::Yes,
            price_cents,
            liquidity_usd: liquidity,
            fetched_at: Utc::now(),
        };

        Ok((quote, status))
    }

    /// Fetch top-of-book YES liquidity (sum of top 3 levels in USD).
    #[instrument(skip(self))]
    pub async fn get_yes_liquidity(&self, ticker: &str) -> Result<Decimal, PkError> {
        let url = format!("{}/markets/{}/orderbook", self.base, ticker);
        let mut req = self.http.get(&url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let resp: KalshiOrderbookResp = req.send().await?.json().await?;
        let top3: Decimal = resp
            .orderbook
            .yes
            .iter()
            .take(3)
            .map(|level| {
                let px = level[0] / 100.0;  // cents → dollars
                let qty = level[1];
                Decimal::from_str(&format!("{:.4}", px * qty)).unwrap_or(Decimal::ZERO)
            })
            .fold(Decimal::ZERO, |acc, v| acc + v);

        Ok(top3)
    }
}
