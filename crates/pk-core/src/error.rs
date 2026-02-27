use thiserror::Error;

#[derive(Error, Debug)]
pub enum PkError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Kalshi API error: {msg}")]
    Kalshi { msg: String },

    #[error("Polymarket API error: {msg}")]
    Polymarket { msg: String },

    #[error("Market not found: {id}")]
    MarketNotFound { id: String },

    #[error("No liquidity available for {side} on {market}")]
    NoLiquidity { market: String, side: String },

    #[error("Order rejected: {reason}")]
    OrderRejected { reason: String },

    #[error("Rate limit hit — retry after {retry_ms}ms")]
    RateLimit { retry_ms: u64 },

    #[error("Config error: {0}")]
    Config(String),
}
