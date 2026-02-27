use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Which side of a binary market
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketSide {
    Yes,
    No,
}

impl std::fmt::Display for MarketSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "YES"),
            Self::No => write!(f, "NO"),
        }
    }
}

/// A single price quote from one exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub exchange: String,
    pub side: MarketSide,
    /// Price in cents (0–100)
    pub price_cents: Decimal,
    /// Best available liquidity in USD at this price
    pub liquidity_usd: Decimal,
    pub fetched_at: DateTime<Utc>,
}

/// Snapshot of both exchanges for the same BTC 15-min market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcMarketSnapshot {
    pub kalshi_ticker: String,
    pub polymarket_token_yes: String,
    pub kalshi_yes: Option<PriceQuote>,
    pub kalshi_status: KalshiStatus,
    pub polymarket_yes: Option<PriceQuote>,
    pub polymarket_no: Option<PriceQuote>,
    pub market_start: DateTime<Utc>,
    pub snapshot_at: DateTime<Utc>,
    /// Seconds elapsed since market start
    pub elapsed_secs: i64,
}

impl BtcMarketSnapshot {
    pub fn spread_cents(&self) -> Option<Decimal> {
        let k = self.kalshi_yes.as_ref()?.price_cents;
        let p = self.polymarket_yes.as_ref()?.price_cents;
        Some(k - p)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KalshiStatus {
    Open,
    Closed,
    Settled,
    Unknown,
}

impl std::fmt::Display for KalshiStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open     => write!(f, "open"),
            Self::Closed   => write!(f, "closed"),
            Self::Settled  => write!(f, "settled"),
            Self::Unknown  => write!(f, "unknown"),
        }
    }
}

/// What kind of arbitrage signal was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    /// Kalshi YES in target range AND Polymarket at least N¢ cheaper → buy Polymarket
    SpreadArb,
    /// Kalshi finished but Polymarket still open → buy Polymarket
    LateResolution,
    /// No signal
    None,
}

/// Full arbitrage signal emitted by the signal engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSignal {
    pub kind: SignalKind,
    pub kalshi_yes_cents: Option<Decimal>,
    pub polymarket_yes_cents: Option<Decimal>,
    pub spread_cents: Option<Decimal>,
    pub kalshi_status: KalshiStatus,
    pub start_window_passed: bool,
    pub signal_at: DateTime<Utc>,
    /// Human-readable reason string
    pub reason: String,
}

impl ArbitrageSignal {
    pub fn none(start_window_passed: bool, reason: impl Into<String>) -> Self {
        Self {
            kind: SignalKind::None,
            kalshi_yes_cents: None,
            polymarket_yes_cents: None,
            spread_cents: None,
            kalshi_status: KalshiStatus::Unknown,
            start_window_passed,
            signal_at: Utc::now(),
            reason: reason.into(),
        }
    }

    pub fn is_actionable(&self) -> bool {
        self.kind != SignalKind::None && self.start_window_passed
    }
}
