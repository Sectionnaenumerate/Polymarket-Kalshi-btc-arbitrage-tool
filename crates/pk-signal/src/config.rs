use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// All tunable parameters for the arbitrage signal engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    // ── Market identification ─────────────────────────────────────────────────
    pub kalshi_ticker: String,
    pub polymarket_token_yes: String,
    pub polymarket_token_no: Option<String>,

    // ── Timing ────────────────────────────────────────────────────────────────
    /// When the 15-minute BTC market opens (ISO 8601)
    pub market_start: DateTime<Utc>,
    /// Do not emit signals until this many minutes after market start
    pub start_delay_mins: u64,

    // ── Spread rule ───────────────────────────────────────────────────────────
    /// Minimum Kalshi YES price (cents) to activate spread rule
    pub kalshi_min_cents: Decimal,
    /// Maximum Kalshi YES price (cents) to activate spread rule
    pub kalshi_max_cents: Decimal,
    /// Minimum spread (Kalshi − Polymarket, cents) required to signal
    pub min_spread_cents: Decimal,

    // ── Execution ─────────────────────────────────────────────────────────────
    /// USD amount per buy order on Polymarket
    pub trade_usd: Decimal,
    /// Minimum seconds between consecutive buy orders
    pub buy_cooldown_secs: u64,
    /// If false, only log signals — do not place real orders
    pub trading_enabled: bool,
}

impl SignalConfig {
    /// Build from environment variables (mirrors .env.example keys).
    pub fn from_env() -> Result<Self, String> {
        let get = |k: &str| std::env::var(k).map_err(|_| format!("missing env: {k}"));
        let dec = |k: &str| -> Result<Decimal, String> {
            Decimal::from_str(&get(k)?).map_err(|e| format!("{k}: {e}"))
        };

        Ok(Self {
            kalshi_ticker: get("KALSHI_TICKER")?,
            polymarket_token_yes: get("POLYMARKET_TOKEN_YES")?,
            polymarket_token_no: std::env::var("POLYMARKET_TOKEN_NO").ok(),
            market_start: get("MARKET_START_TIME")?
                .parse::<DateTime<Utc>>()
                .map_err(|e| format!("MARKET_START_TIME: {e}"))?,
            start_delay_mins: std::env::var("START_DELAY_MINS")
                .unwrap_or_else(|_| "8".into())
                .parse()
                .unwrap_or(8),
            kalshi_min_cents: dec("KALSHI_MIN_CENTS").unwrap_or(Decimal::from(93)),
            kalshi_max_cents: dec("KALSHI_MAX_CENTS").unwrap_or(Decimal::from(96)),
            min_spread_cents: dec("MIN_SPREAD_CENTS").unwrap_or(Decimal::from(10)),
            trade_usd: dec("TRADE_USD").unwrap_or(Decimal::from(10)),
            buy_cooldown_secs: std::env::var("BUY_COOLDOWN_SECS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .unwrap_or(60),
            trading_enabled: std::env::var("POLYMARKET_PRIVATE_KEY").is_ok(),
        })
    }
}
