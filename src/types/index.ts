export type MarketSide = 'YES' | 'NO';
export type KalshiStatus = 'open' | 'closed' | 'settled' | 'unknown';
export type SignalKind = 'spread_arb' | 'late_resolution' | 'none';

export interface PriceQuote {
  exchange:      'kalshi' | 'polymarket';
  side:          MarketSide;
  priceCents:    number;      // 0–100
  liquidityUsd:  number;
  fetchedAt:     Date;
}

export interface BtcMarketSnapshot {
  kalshiTicker:        string;
  polymarketTokenYes:  string;
  kalshiYes:           PriceQuote | null;
  kalshiStatus:        KalshiStatus;
  polymarketYes:       PriceQuote | null;
  polymarketNo:        PriceQuote | null;
  marketStart:         Date;
  snapshotAt:          Date;
  elapsedSecs:         number;
  spreadCents:         number | null;
}

export interface ArbitrageSignal {
  kind:               SignalKind;
  kalshiYesCents:     number | null;
  polymarketYesCents: number | null;
  spreadCents:        number | null;
  kalshiStatus:       KalshiStatus;
  startWindowPassed:  boolean;
  signalAt:           Date;
  reason:             string;
  actionable:         boolean;
}

export interface BotConfig {
  port:               number;
  pollIntervalMs:     number;
  marketStartTime:    Date;
  startDelayMins:     number;
  kalshiApiBase:      string;
  kalshiTicker:       string;
  polymarketClobBase: string;
  polymarketTokenYes: string;
  polymarketTokenNo?: string;
  kalshiMinCents:     number;
  kalshiMaxCents:     number;
  minSpreadCents:     number;
  tradeUsd:           number;
  buyCooldownSecs:    number;
  tradingEnabled:     boolean;
}

export interface BotStatus {
  pollingActive:       boolean;
  tradingEnabled:      boolean;
  totalSignals:        number;
  totalOrdersPlaced:   number;
  lastSnapshot:        BtcMarketSnapshot | null;
  lastSignal:          ArbitrageSignal | null;
}
