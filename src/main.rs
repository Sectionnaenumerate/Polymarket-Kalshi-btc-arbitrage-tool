use anyhow::Result;
use dotenv::dotenv;
use tracing_subscriber::{fmt, EnvFilter};

mod api;
mod poller;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("pk_arb=info,pk_signal=info")),
        )
        .without_time()
        .with_target(false)
        .init();

    let cfg = pk_signal::SignalConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Config error: {e}"))?;

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap_or(3000);

    let poll_ms: u64 = std::env::var("POLL_INTERVAL_MS")
        .unwrap_or_else(|_| "5000".into())
        .parse()
        .unwrap_or(5000);

    tracing::info!("🚀 Polymarket-Kalshi BTC Arbitrage Bot starting");
    tracing::info!("   Kalshi ticker:  {}", cfg.kalshi_ticker);
    tracing::info!("   Poly token YES: {}", cfg.polymarket_token_yes);
    tracing::info!("   Market start:   {}", cfg.market_start);
    tracing::info!("   Trading:        {}", if cfg.trading_enabled { "ENABLED" } else { "DISABLED (signal-only)" });
    tracing::info!("   Poll interval:  {}ms", poll_ms);
    tracing::info!("   API port:       {}", port);

    let shared = state::AppState::new(cfg.clone(), poll_ms);

    // Start HTTP API and price poller concurrently
    tokio::try_join!(
        api::serve(shared.clone(), port),
        poller::run(shared, cfg, poll_ms),
    )?;

    Ok(())
}
