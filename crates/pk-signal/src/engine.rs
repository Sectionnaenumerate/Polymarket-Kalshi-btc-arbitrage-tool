use chrono::Utc;
use pk_core::{ArbitrageSignal, BtcMarketSnapshot, KalshiStatus, SignalKind};
use rust_decimal::Decimal;
use tracing::{debug, info, warn};

use crate::config::SignalConfig;

pub struct SignalEngine {
    pub cfg: SignalConfig,
}

impl SignalEngine {
    pub fn new(cfg: SignalConfig) -> Self {
        Self { cfg }
    }

    /// Evaluate a fresh market snapshot and return a signal.
    pub fn evaluate(&self, snap: &BtcMarketSnapshot) -> ArbitrageSignal {
        let start_window_passed = self.start_window_passed(snap.elapsed_secs);

        if !start_window_passed {
            let remaining = self.cfg.start_delay_mins as i64 * 60 - snap.elapsed_secs;
            debug!("Start window not passed — {remaining}s remaining");
            return ArbitrageSignal::none(false, format!("Waiting for start window ({remaining}s)"));
        }

        // ── Rule 2: Late resolution — Kalshi finished, Polymarket still open ─
        if matches!(snap.kalshi_status, KalshiStatus::Closed | KalshiStatus::Settled) {
            let has_poly_liquidity = snap
                .polymarket_yes
                .as_ref()
                .map(|q| q.liquidity_usd > Decimal::ZERO)
                .unwrap_or(false);

            if has_poly_liquidity {
                info!(
                    kind = "late_resolution",
                    kalshi_status = %snap.kalshi_status,
                    "Late-resolution arb signal"
                );
                return ArbitrageSignal {
                    kind: SignalKind::LateResolution,
                    kalshi_yes_cents: snap.kalshi_yes.as_ref().map(|q| q.price_cents),
                    polymarket_yes_cents: snap.polymarket_yes.as_ref().map(|q| q.price_cents),
                    spread_cents: snap.spread_cents(),
                    kalshi_status: snap.kalshi_status,
                    start_window_passed: true,
                    signal_at: Utc::now(),
                    reason: format!(
                        "Kalshi {} but Polymarket still open — timing arb",
                        snap.kalshi_status
                    ),
                };
            }
        }

        // ── Rule 1: Spread rule ───────────────────────────────────────────────
        let k_price = match snap.kalshi_yes.as_ref().map(|q| q.price_cents) {
            Some(p) => p,
            None => {
                warn!("No Kalshi price in snapshot");
                return ArbitrageSignal::none(true, "No Kalshi price available");
            }
        };

        let p_price = match snap.polymarket_yes.as_ref().map(|q| q.price_cents) {
            Some(p) => p,
            None => {
                warn!("No Polymarket price in snapshot");
                return ArbitrageSignal::none(true, "No Polymarket price available");
            }
        };

        let spread = k_price - p_price;

        let in_kalshi_range = k_price >= self.cfg.kalshi_min_cents
            && k_price <= self.cfg.kalshi_max_cents;
        let spread_sufficient = spread >= self.cfg.min_spread_cents;

        debug!(
            k = %k_price, p = %p_price, spread = %spread,
            in_range = in_kalshi_range, sufficient = spread_sufficient,
            "Spread evaluation"
        );

        if in_kalshi_range && spread_sufficient {
            info!(
                kind = "spread_arb",
                kalshi = %k_price,
                polymarket = %p_price,
                spread = %spread,
                "Spread arb signal"
            );
            ArbitrageSignal {
                kind: SignalKind::SpreadArb,
                kalshi_yes_cents: Some(k_price),
                polymarket_yes_cents: Some(p_price),
                spread_cents: Some(spread),
                kalshi_status: snap.kalshi_status,
                start_window_passed: true,
                signal_at: Utc::now(),
                reason: format!(
                    "Kalshi={k_price}¢ in [{}-{}¢], Polymarket={p_price}¢, spread={spread}¢ ≥ {}¢",
                    self.cfg.kalshi_min_cents,
                    self.cfg.kalshi_max_cents,
                    self.cfg.min_spread_cents
                ),
            }
        } else {
            ArbitrageSignal::none(
                true,
                format!(
                    "No signal — Kalshi={k_price}¢, Polymarket={p_price}¢, spread={spread}¢ \
                     (need Kalshi in [{}-{}¢] and spread≥{}¢)",
                    self.cfg.kalshi_min_cents,
                    self.cfg.kalshi_max_cents,
                    self.cfg.min_spread_cents
                ),
            )
        }
    }

    fn start_window_passed(&self, elapsed_secs: i64) -> bool {
        elapsed_secs >= self.cfg.start_delay_mins as i64 * 60
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use pk_core::{BtcMarketSnapshot, MarketSide, PriceQuote};
    use rust_decimal_macros::dec;

    fn make_cfg() -> SignalConfig {
        SignalConfig {
            kalshi_ticker: "KXBTC-TEST".into(),
            polymarket_token_yes: "0xabc".into(),
            polymarket_token_no: None,
            market_start: Utc::now(),
            start_delay_mins: 0, // no delay for tests
            kalshi_min_cents: dec!(93),
            kalshi_max_cents: dec!(96),
            min_spread_cents: dec!(10),
            trade_usd: dec!(10),
            buy_cooldown_secs: 60,
            trading_enabled: false,
        }
    }

    fn quote(exchange: &str, side: MarketSide, cents: Decimal) -> PriceQuote {
        PriceQuote {
            exchange: exchange.into(),
            side,
            price_cents: cents,
            liquidity_usd: dec!(500),
            fetched_at: Utc::now(),
        }
    }

    fn snap(k_cents: Decimal, p_cents: Decimal, status: KalshiStatus) -> BtcMarketSnapshot {
        BtcMarketSnapshot {
            kalshi_ticker: "KXBTC-TEST".into(),
            polymarket_token_yes: "0xabc".into(),
            kalshi_yes: Some(quote("kalshi", MarketSide::Yes, k_cents)),
            kalshi_status: status,
            polymarket_yes: Some(quote("polymarket", MarketSide::Yes, p_cents)),
            polymarket_no: None,
            market_start: Utc::now(),
            snapshot_at: Utc::now(),
            elapsed_secs: 600, // 10 minutes
        }
    }

    #[test]
    fn spread_arb_fires_when_conditions_met() {
        let engine = SignalEngine::new(make_cfg());
        let s = snap(dec!(95), dec!(82), KalshiStatus::Open); // spread = 13¢ ≥ 10¢
        let sig = engine.evaluate(&s);
        assert_eq!(sig.kind, SignalKind::SpreadArb);
        assert!(sig.is_actionable());
    }

    #[test]
    fn spread_arb_no_fire_when_spread_too_small() {
        let engine = SignalEngine::new(make_cfg());
        let s = snap(dec!(94), dec!(88), KalshiStatus::Open); // spread = 6¢ < 10¢
        let sig = engine.evaluate(&s);
        assert_eq!(sig.kind, SignalKind::None);
    }

    #[test]
    fn spread_arb_no_fire_kalshi_out_of_range() {
        let engine = SignalEngine::new(make_cfg());
        let s = snap(dec!(80), dec!(68), KalshiStatus::Open); // kalshi < 93¢
        let sig = engine.evaluate(&s);
        assert_eq!(sig.kind, SignalKind::None);
    }

    #[test]
    fn late_resolution_fires_when_kalshi_closed() {
        let engine = SignalEngine::new(make_cfg());
        let s = snap(dec!(99), dec!(72), KalshiStatus::Closed);
        let sig = engine.evaluate(&s);
        assert_eq!(sig.kind, SignalKind::LateResolution);
    }

    #[test]
    fn no_signal_before_start_window() {
        let mut cfg = make_cfg();
        cfg.start_delay_mins = 8;
        let engine = SignalEngine::new(cfg);
        let mut s = snap(dec!(95), dec!(82), KalshiStatus::Open);
        s.elapsed_secs = 300; // only 5 minutes
        let sig = engine.evaluate(&s);
        assert_eq!(sig.kind, SignalKind::None);
        assert!(!sig.start_window_passed);
    }
}
