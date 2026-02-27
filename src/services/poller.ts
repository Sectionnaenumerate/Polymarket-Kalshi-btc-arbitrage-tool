import { KalshiClient }  from '../clients/kalshi';
import { PolyClient }    from '../clients/polymarket';
import { SignalService } from './signal';
import { BotConfig, BotStatus, BtcMarketSnapshot } from '../types';
import { logger }        from '../utils/logger';

export class PollerService {
  private kalshi:   KalshiClient;
  private poly:     PolyClient;
  private signal:   SignalService;
  private timer:    NodeJS.Timeout | null = null;
  private lastBuyAt: number | null = null;

  public status: BotStatus = {
    pollingActive:     false,
    tradingEnabled:    false,
    totalSignals:      0,
    totalOrdersPlaced: 0,
    lastSnapshot:      null,
    lastSignal:        null,
  };

  constructor(private cfg: BotConfig) {
    this.kalshi  = new KalshiClient(cfg.kalshiApiBase);
    this.poly    = new PolyClient(
      cfg.polymarketClobBase,
      cfg.tradingEnabled ? process.env.POLYMARKET_PRIVATE_KEY : undefined
    );
    this.signal  = new SignalService(cfg);
    this.status.tradingEnabled = cfg.tradingEnabled;
  }

  start(): void {
    if (this.status.pollingActive) return;
    this.status.pollingActive = true;
    this.tick();
    logger.info(`Polling started (interval: ${this.cfg.pollIntervalMs}ms)`);
  }

  stop(): void {
    this.status.pollingActive = false;
    if (this.timer) clearTimeout(this.timer);
    logger.info('Polling stopped');
  }

  private tick(): void {
    if (!this.status.pollingActive) return;
    this.timer = setTimeout(async () => {
      try {
        await this.poll();
      } catch (err) {
        logger.error('Poll error:', err);
      } finally {
        this.tick();
      }
    }, this.cfg.pollIntervalMs);
  }

  private async poll(): Promise<void> {
    const snap = await this.fetchSnapshot();
    const sig  = this.signal.evaluate(snap);

    this.status.lastSnapshot = snap;
    this.status.lastSignal   = sig;

    if (sig.kind !== 'none') {
      this.status.totalSignals++;
      logger.info(
        `🔔 SIGNAL [${sig.kind}] Kalshi=${sig.kalshiYesCents}¢ Poly=${sig.polymarketYesCents}¢ ` +
        `spread=${sig.spreadCents}¢ — ${sig.reason}`
      );
    }

    if (sig.actionable && this.cfg.tradingEnabled) {
      const now = Date.now();
      const cooldownOk = !this.lastBuyAt || (now - this.lastBuyAt) / 1000 >= this.cfg.buyCooldownSecs;
      if (cooldownOk) {
        try {
          const orderId = await this.poly.placeBuy(this.cfg.polymarketTokenYes, this.cfg.tradeUsd);
          logger.info(`✅ Order placed: ${orderId}`);
          this.status.totalOrdersPlaced++;
          this.lastBuyAt = now;
        } catch (err) {
          logger.error('Order failed:', err);
        }
      } else {
        logger.info('⏳ Buy cooldown active — skipping order');
      }
    }
  }

  private async fetchSnapshot(): Promise<BtcMarketSnapshot> {
    const [{ quote: kalshiYes, status: kalshiStatus }, polyYes] = await Promise.all([
      this.kalshi.getBtcPrice(this.cfg.kalshiTicker),
      this.poly.getPrice(this.cfg.polymarketTokenYes, 'YES'),
    ]);

    const polyNo = this.cfg.polymarketTokenNo
      ? await this.poly.getPrice(this.cfg.polymarketTokenNo, 'NO')
      : null;

    const elapsed = Math.floor((Date.now() - this.cfg.marketStartTime.getTime()) / 1000);
    const kCents  = kalshiYes.priceCents;
    const pCents  = polyYes.priceCents;

    return {
      kalshiTicker:        this.cfg.kalshiTicker,
      polymarketTokenYes:  this.cfg.polymarketTokenYes,
      kalshiYes,
      kalshiStatus,
      polymarketYes: polyYes,
      polymarketNo:  polyNo,
      marketStart:   this.cfg.marketStartTime,
      snapshotAt:    new Date(),
      elapsedSecs:   elapsed,
      spreadCents:   kCents != null && pCents != null ? kCents - pCents : null,
    };
  }
}
