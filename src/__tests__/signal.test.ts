import { SignalService } from '../services/signal';
import { BotConfig, BtcMarketSnapshot, PriceQuote } from '../types';

const baseCfg: BotConfig = {
  port: 3000,
  pollIntervalMs: 5000,
  marketStartTime: new Date(),
  startDelayMins: 0,           // no delay for tests
  kalshiApiBase: '',
  kalshiTicker: 'KXBTC-TEST',
  polymarketClobBase: '',
  polymarketTokenYes: '0xtest',
  kalshiMinCents: 93,
  kalshiMaxCents: 96,
  minSpreadCents: 10,
  tradeUsd: 10,
  buyCooldownSecs: 60,
  tradingEnabled: false,
};

function quote(exchange: 'kalshi' | 'polymarket', cents: number): PriceQuote {
  return {
    exchange,
    side: 'YES',
    priceCents: cents,
    liquidityUsd: 500,
    fetchedAt: new Date(),
  };
}

function snap(kCents: number, pCents: number, status: BtcMarketSnapshot['kalshiStatus']): BtcMarketSnapshot {
  return {
    kalshiTicker: 'KXBTC-TEST',
    polymarketTokenYes: '0xtest',
    kalshiYes: quote('kalshi', kCents),
    kalshiStatus: status,
    polymarketYes: quote('polymarket', pCents),
    polymarketNo: null,
    marketStart: new Date(),
    snapshotAt: new Date(),
    elapsedSecs: 600,
    spreadCents: kCents - pCents,
  };
}

describe('SignalService', () => {
  const svc = new SignalService(baseCfg);

  test('fires spread_arb when Kalshi in range and spread sufficient', () => {
    const sig = svc.evaluate(snap(95, 82, 'open')); // spread = 13¢
    expect(sig.kind).toBe('spread_arb');
    expect(sig.actionable).toBe(true);
    expect(sig.spreadCents).toBe(13);
  });

  test('no signal when spread too small', () => {
    const sig = svc.evaluate(snap(94, 88, 'open')); // spread = 6¢ < 10¢
    expect(sig.kind).toBe('none');
  });

  test('no signal when Kalshi below min range', () => {
    const sig = svc.evaluate(snap(80, 68, 'open')); // kalshi < 93¢
    expect(sig.kind).toBe('none');
  });

  test('no signal when Kalshi above max range', () => {
    const sig = svc.evaluate(snap(98, 80, 'open')); // kalshi > 96¢
    expect(sig.kind).toBe('none');
  });

  test('fires late_resolution when Kalshi closed and Polymarket open', () => {
    const sig = svc.evaluate(snap(99, 72, 'closed'));
    expect(sig.kind).toBe('late_resolution');
    expect(sig.actionable).toBe(true);
  });

  test('fires late_resolution when Kalshi settled', () => {
    const sig = svc.evaluate(snap(99, 72, 'settled'));
    expect(sig.kind).toBe('late_resolution');
  });

  test('no signal before start window', () => {
    const cfg = { ...baseCfg, startDelayMins: 8 };
    const s = new SignalService(cfg);
    const earlySnap = { ...snap(95, 82, 'open'), elapsedSecs: 300 };
    const sig = s.evaluate(earlySnap);
    expect(sig.kind).toBe('none');
    expect(sig.startWindowPassed).toBe(false);
  });

  test('signal after start window passes', () => {
    const cfg = { ...baseCfg, startDelayMins: 8 };
    const s = new SignalService(cfg);
    const lateSnap = { ...snap(95, 82, 'open'), elapsedSecs: 490 }; // 8m10s
    const sig = s.evaluate(lateSnap);
    expect(sig.kind).toBe('spread_arb');
    expect(sig.startWindowPassed).toBe(true);
  });
});
