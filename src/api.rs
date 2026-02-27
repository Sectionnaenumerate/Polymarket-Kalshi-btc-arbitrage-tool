use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::info;

use crate::state::AppState;

pub async fn serve(state: AppState, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/health",      get(health))
        .route("/status",      get(status))
        .route("/poll/start",  post(poll_start))
        .route("/poll/stop",   post(poll_stop))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP API listening on http://0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── GET /health ───────────────────────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "polymarket-kalshi-btc-arbitrage-tool" }))
}

// ── GET /status ───────────────────────────────────────────────────────────────

async fn status(State(state): State<AppState>) -> Json<Value> {
    let s = state.read().await;
    let bot = &s.state;

    Json(json!({
        "polling_active": bot.polling_active,
        "trading_enabled": s.cfg.trading_enabled,
        "total_signals": bot.total_signals,
        "total_orders_placed": bot.total_orders_placed,
        "market_config": {
            "kalshi_ticker": s.cfg.kalshi_ticker,
            "polymarket_token_yes": s.cfg.polymarket_token_yes,
            "market_start": s.cfg.market_start,
            "start_delay_mins": s.cfg.start_delay_mins,
            "kalshi_range_cents": [s.cfg.kalshi_min_cents, s.cfg.kalshi_max_cents],
            "min_spread_cents": s.cfg.min_spread_cents,
            "trade_usd": s.cfg.trade_usd,
            "buy_cooldown_secs": s.cfg.buy_cooldown_secs,
        },
        "last_snapshot": bot.last_snapshot.as_ref().map(|snap| json!({
            "kalshi_yes_cents": snap.kalshi_yes.as_ref().map(|q| q.price_cents),
            "kalshi_status": snap.kalshi_status,
            "polymarket_yes_cents": snap.polymarket_yes.as_ref().map(|q| q.price_cents),
            "spread_cents": snap.spread_cents(),
            "elapsed_secs": snap.elapsed_secs,
            "snapshot_at": snap.snapshot_at,
        })),
        "last_signal": bot.last_signal.as_ref().map(|sig| json!({
            "kind": sig.kind,
            "actionable": sig.is_actionable(),
            "kalshi_yes_cents": sig.kalshi_yes_cents,
            "polymarket_yes_cents": sig.polymarket_yes_cents,
            "spread_cents": sig.spread_cents,
            "start_window_passed": sig.start_window_passed,
            "reason": sig.reason,
            "signal_at": sig.signal_at,
        })),
    }))
}

// ── POST /poll/start ──────────────────────────────────────────────────────────

async fn poll_start(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let mut s = state.write().await;
    s.state.polling_active = true;
    info!("Polling started via API");
    (StatusCode::OK, Json(json!({ "polling_active": true })))
}

// ── POST /poll/stop ───────────────────────────────────────────────────────────

async fn poll_stop(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let mut s = state.write().await;
    s.state.polling_active = false;
    info!("Polling stopped via API");
    (StatusCode::OK, Json(json!({ "polling_active": false })))
}
