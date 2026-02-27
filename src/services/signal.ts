import {
  ArbitrageSignal,
  BotConfig,
  BtcMarketSnapshot,
  SignalKind,
} from '../types';

export class SignalService {
  constructor(private cfg: BotConfig) {}

  evaluate(snap: BtcMarketSnapshot): ArbitrageSignal {
    const startWindowPassed = snap.elapsedSecs >= this.cfg.startDelayMins * 60;

    if (!startWindowPassed) {
      const remaining = this.cfg.startDelayMins * 60 - snap.elapsedSecs;
      return this.none(false, `Waiting for start window (${remaining}s remaining)`);
    }

    // ── Rule 2: Late resolution ──────────────────────────────────────────────
    if (snap.kalshiStatus === 'closed' || snap.kalshiStatus === 'settled') {
      const hasLiquidity = (snap.polymarketYes?.liquidityUsd ?? 0) > 0;
      if (hasLiquidity) {
        return {
          kind: 'late_resolution',
          kalshiYesCents: snap.kalshiYes?.priceCents ?? null,
          polymarketYesCents: snap.polymarketYes?.priceCents ?? null,
          spreadCents: snap.spreadCents,
          kalshiStatus: snap.kalshiStatus,
          startWindowPassed: true,
          signalAt: new Date(),
          reason: `Kalshi ${snap.kalshiStatus} but Polymarket still open — timing arb`,
          actionable: true,
        };
      }
    }

    // ── Rule 1: Spread rule ───────────────────────────────────────────────────
    const kCents = snap.kalshiYes?.priceCents;
    const pCents = snap.polymarketYes?.priceCents;

    if (kCents == null || pCents == null) {
      return this.none(true, 'Missing price data');
    }

    const spread = kCents - pCents;
    const inKalshiRange = kCents >= this.cfg.kalshiMinCents && kCents <= this.cfg.kalshiMaxCents;
    const spreadOk = spread >= this.cfg.minSpreadCents;

    if (inKalshiRange && spreadOk) {
      return {
        kind: 'spread_arb',
        kalshiYesCents: kCents,
        polymarketYesCents: pCents,
        spreadCents: spread,
        kalshiStatus: snap.kalshiStatus,
        startWindowPassed: true,
        signalAt: new Date(),
        reason: `Kalshi=${kCents}¢ in [${this.cfg.kalshiMinCents}–${this.cfg.kalshiMaxCents}¢], ` +
                `Polymarket=${pCents}¢, spread=${spread}¢ ≥ ${this.cfg.minSpreadCents}¢`,
        actionable: true,
      };
    }

    return this.none(
      true,
      `No signal — Kalshi=${kCents}¢, Poly=${pCents}¢, spread=${spread}¢ ` +
      `(need Kalshi in [${this.cfg.kalshiMinCents}–${this.cfg.kalshiMaxCents}¢] ` +
      `and spread≥${this.cfg.minSpreadCents}¢)`
    );
  }

  private none(startWindowPassed: boolean, reason: string): ArbitrageSignal {
    return {
      kind: 'none',
      kalshiYesCents: null,
      polymarketYesCents: null,
      spreadCents: null,
      kalshiStatus: 'unknown',
      startWindowPassed,
      signalAt: new Date(),
      reason,
      actionable: false,
    };
  }
}
