use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{debug, instrument};

use crate::{
    error::PkError,
    types::{MarketSide, PriceQuote},
};

const DEFAULT_CLOB: &str = "https://clob.polymarket.com";

pub struct PolyClient {
    http: Client,
    clob_base: String,
}

// ─── Raw API shapes ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ClobPriceResp {
    price: String,
}

#[derive(Debug, Deserialize)]
struct ClobOrderbookResp {
    bids: Vec<ClobLevel>,
    asks: Vec<ClobLevel>,
}

#[derive(Debug, Deserialize)]
struct ClobLevel {
    price: String,
    size: String,
}

#[derive(Debug, Deserialize)]
struct ClobMarketResp {
    tokens: Vec<ClobToken>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ClobToken {
    token_id: String,
    outcome: String,
}

// ─── Client ──────────────────────────────────────────────────────────────────

impl PolyClient {
    pub fn new(clob_base: Option<String>) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            clob_base: clob_base.unwrap_or_else(|| DEFAULT_CLOB.to_string()),
        }
    }

    /// Get the current mid-price for a YES token (returns cents 0–100).
    #[instrument(skip(self))]
    pub async fn get_yes_price(&self, token_id: &str) -> Result<PriceQuote, PkError> {
        let url = format!("{}/price?token_id={}&side=buy", self.clob_base, token_id);
        debug!("GET {url}");

        let resp: ClobPriceResp = self.http.get(&url).send().await?.json().await?;
        let price_frac = Decimal::from_str(&resp.price)
            .map_err(|e| PkError::Polymarket { msg: e.to_string() })?;
        // CLOB returns price as 0–1 fraction; convert to cents
        let price_cents = price_frac * Decimal::from(100);

        Ok(PriceQuote {
            exchange: "polymarket".to_string(),
            side: MarketSide::Yes,
            price_cents,
            liquidity_usd: Decimal::ZERO, // filled separately
            fetched_at: Utc::now(),
        })
    }

    /// Get the current mid-price for a NO token (returns cents 0–100).
    #[instrument(skip(self))]
    pub async fn get_no_price(&self, token_id: &str) -> Result<PriceQuote, PkError> {
        let url = format!("{}/price?token_id={}&side=buy", self.clob_base, token_id);
        let resp: ClobPriceResp = self.http.get(&url).send().await?.json().await?;
        let price_frac = Decimal::from_str(&resp.price)
            .map_err(|e| PkError::Polymarket { msg: e.to_string() })?;
        let price_cents = price_frac * Decimal::from(100);

        Ok(PriceQuote {
            exchange: "polymarket".to_string(),
            side: MarketSide::No,
            price_cents,
            liquidity_usd: Decimal::ZERO,
            fetched_at: Utc::now(),
        })
    }

    /// Compute available liquidity (sum of top 5 bid levels in USD).
    #[instrument(skip(self))]
    pub async fn get_liquidity(&self, token_id: &str) -> Result<Decimal, PkError> {
        let url = format!("{}/book?token_id={}", self.clob_base, token_id);
        let resp: ClobOrderbookResp = self.http.get(&url).send().await?.json().await?;

        let liquidity = resp
            .bids
            .iter()
            .take(5)
            .filter_map(|level| {
                let px = Decimal::from_str(&level.price).ok()?;
                let sz = Decimal::from_str(&level.size).ok()?;
                Some(px * sz)
            })
            .fold(Decimal::ZERO, |acc, v| acc + v);

        Ok(liquidity)
    }

    /// Place a market buy order on Polymarket CLOB.
    /// Requires a pre-signed payload from pk-signer.
    #[instrument(skip(self, signed_payload))]
    pub async fn place_buy(
        &self,
        token_id: &str,
        amount_usd: Decimal,
        signed_payload: serde_json::Value,
    ) -> Result<String, PkError> {
        let url = format!("{}/order", self.clob_base);
        let resp: serde_json::Value = self
            .http
            .post(&url)
            .json(&signed_payload)
            .send()
            .await?
            .json()
            .await?;

        let order_id = resp["orderID"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        if resp["status"] == "matched" || resp["status"] == "live" {
            Ok(order_id)
        } else {
            Err(PkError::OrderRejected {
                reason: resp["errorMsg"].as_str().unwrap_or("unknown").to_string(),
            })
        }
    }
}
