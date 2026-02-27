use pk_core::{ArbitrageSignal, BtcMarketSnapshot};
use pk_signal::SignalConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct BotState {
    pub last_snapshot: Option<BtcMarketSnapshot>,
    pub last_signal: Option<ArbitrageSignal>,
    pub polling_active: bool,
    pub total_signals: u64,
    pub total_orders_placed: u64,
}

pub type AppState = Arc<RwLock<BotStateInner>>;

pub struct BotStateInner {
    pub state: BotState,
    pub cfg: SignalConfig,
    pub poll_interval_ms: u64,
}

impl BotStateInner {
    pub fn new(cfg: SignalConfig, poll_interval_ms: u64) -> Self {
        Self {
            state: BotState { polling_active: true, ..Default::default() },
            cfg,
            poll_interval_ms,
        }
    }
}

pub fn new_state(cfg: SignalConfig, poll_interval_ms: u64) -> AppState {
    Arc::new(RwLock::new(BotStateInner::new(cfg, poll_interval_ms)))
}

// Re-export the constructor as AppState::new for ergonomics
pub trait AppStateExt {
    fn new(cfg: SignalConfig, poll_interval_ms: u64) -> Self;
}

impl AppStateExt for AppState {
    fn new(cfg: SignalConfig, poll_interval_ms: u64) -> Self {
        new_state(cfg, poll_interval_ms)
    }
}
