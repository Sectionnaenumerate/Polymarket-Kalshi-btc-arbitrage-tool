use anyhow::Result;
use chrono::Utc;
use pk_core::{BtcMarketSnapshot, KalshiClient, PolyClient, SignalKind};
use pk_signal::{SignalConfig, SignalEngine};
use pk_signer::{ClobOrder, PolyWallet};
use rust_decimal::Decimal;
use tracing::{error, info, warn};

use crate::state::AppState;

pub async fn run(state: AppState, cfg: SignalConfig, poll_ms: u64) -> Result<()> {
    let kalshi = KalshiClient::new(
        std::env::var("KALSHI_API_BASE").ok(),
        std::env::var("KALSHI_API_TOKEN").ok(),
    );
    let poly = PolyClient::new(std::env::var("POLYMARKET_CLOB_BASE").ok());
    let engine = SignalEngine::new(cfg.clone());

    let wallet = if cfg.trading_enabled {
        match PolyWallet::from_env() {
            Ok(w) => {
                info!("Wallet loaded — EOA: {} | effective: {}", w.address, w.effective_address());
                Some(w)
            }
            Err(e) => {
                warn!("Could not load wallet: {e} — running in signal-only mode");
                None
            }
        }
    } else {
        None
    };

    let mut last_buy_at: Option<i64> = None;

    loop {
        // Check if polling is paused via /poll/stop
        {
            let s = state.read().await;
            if !s.state.polling_active {
                tokio::time::sleep(tokio::time::Duration::from_millis(poll_ms)).await;
                continue;
            }
        }

        match fetch_snapshot(&kalshi, &poly, &cfg).await {
            Ok(snap) => {
                let signal = engine.evaluate(&snap);

                // Update shared state
                {
                    let mut s = state.write().await;
                    s.state.last_snapshot = Some(snap.clone());
                    if signal.kind != SignalKind::None {
                        s.state.total_signals += 1;
                    }
                    s.state.last_signal = Some(signal.clone());
                }

                if signal.is_actionable() {
                    let now = Utc::now().timestamp();
                    let cooldown_ok = last_buy_at
                        .map(|t| now - t >= cfg.buy_cooldown_secs as i64)
                        .unwrap_or(true);

                    info!(
                        kind = ?signal.kind,
                        kalshi = ?signal.kalshi_yes_cents,
                        poly = ?signal.polymarket_yes_cents,
                        spread = ?signal.spread_cents,
                        reason = %signal.reason,
                        "🔔 SIGNAL"
                    );

                    if let Some(w) = &wallet {
                        if cooldown_ok {
                            match place_buy(&poly, w, &cfg, &snap.polymarket_token_yes).await {
                                Ok(order_id) => {
                                    info!("✅ Order placed: {order_id}");
                                    last_buy_at = Some(now);
                                    let mut s = state.write().await;
                                    s.state.total_orders_placed += 1;
                                }
                                Err(e) => error!("Order failed: {e}"),
                            }
                        } else {
                            info!("⏳ Cooldown active — skipping order");
                        }
                    }
                }
            }
            Err(e) => error!("Snapshot fetch failed: {e}"),
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(poll_ms)).await;
    }
}

async fn fetch_snapshot(
    kalshi: &KalshiClient,
    poly: &PolyClient,
    cfg: &SignalConfig,
) -> anyhow::Result<BtcMarketSnapshot> {
    let ((k_quote, k_status), p_yes, p_no_opt) = tokio::try_join!(
        kalshi.get_btc_price(&cfg.kalshi_ticker),
        poly.get_yes_price(&cfg.polymarket_token_yes),
        async {
            if let Some(no_token) = &cfg.polymarket_token_no {
                Ok(Some(poly.get_no_price(no_token).await?))
            } else {
                Ok::<_, pk_core::PkError>(None)
            }
        },
    )?;

    let elapsed = (Utc::now() - cfg.market_start).num_seconds();

    Ok(BtcMarketSnapshot {
        kalshi_ticker: cfg.kalshi_ticker.clone(),
        polymarket_token_yes: cfg.polymarket_token_yes.clone(),
        kalshi_yes: Some(k_quote),
        kalshi_status: k_status,
        polymarket_yes: Some(p_yes),
        polymarket_no: p_no_opt,
        market_start: cfg.market_start,
        snapshot_at: Utc::now(),
        elapsed_secs: elapsed,
    })
}

async fn place_buy(
    poly: &PolyClient,
    wallet: &PolyWallet,
    cfg: &SignalConfig,
    token_id: &str,
) -> anyhow::Result<String> {
    let price_quote = poly.get_yes_price(token_id).await?;
    let price_frac = price_quote.price_cents / Decimal::from(100);
    let size = if price_frac.is_zero() {
        Decimal::ZERO
    } else {
        cfg.trade_usd / price_frac
    };

    let order = ClobOrder::market_buy(token_id, price_frac, size);
    let payload = pk_signer::sign_clob_order(wallet, &order).await?;
    let order_id = poly.place_buy(token_id, cfg.trade_usd, payload).await?;
    Ok(order_id)
}
