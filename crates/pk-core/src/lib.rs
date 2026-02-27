pub mod error;
pub mod kalshi;
pub mod polymarket;
pub mod types;

pub use error::PkError;
pub use kalshi::KalshiClient;
pub use polymarket::PolyClient;
pub use types::{ArbitrageSignal, BtcMarketSnapshot, MarketSide, PriceQuote, SignalKind};
