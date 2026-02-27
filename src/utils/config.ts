import { BotConfig } from '../types';

export function loadConfig(): BotConfig {
  const req = (key: string): string => {
    const v = process.env[key];
    if (!v) throw new Error(`Missing required env var: ${key}`);
    return v;
  };
  const opt = (key: string, fallback: string): string =>
    process.env[key] ?? fallback;

  return {
    port:               parseInt(opt('PORT', '3000')),
    pollIntervalMs:     parseInt(opt('POLL_INTERVAL_MS', '5000')),
    marketStartTime:    new Date(req('MARKET_START_TIME')),
    startDelayMins:     parseInt(opt('START_DELAY_MINS', '8')),
    kalshiApiBase:      opt('KALSHI_API_BASE', 'https://api.elections.kalshi.com/trade-api/v2'),
    kalshiTicker:       req('KALSHI_TICKER'),
    polymarketClobBase: opt('POLYMARKET_CLOB_BASE', 'https://clob.polymarket.com'),
    polymarketTokenYes: req('POLYMARKET_TOKEN_YES'),
    polymarketTokenNo:  process.env.POLYMARKET_TOKEN_NO,
    kalshiMinCents:     parseInt(opt('KALSHI_MIN_CENTS', '93')),
    kalshiMaxCents:     parseInt(opt('KALSHI_MAX_CENTS', '96')),
    minSpreadCents:     parseInt(opt('MIN_SPREAD_CENTS', '10')),
    tradeUsd:           parseFloat(opt('POLYMARKET_TRADE_USD', '10')),
    buyCooldownSecs:    parseInt(opt('POLYMARKET_BUY_COOLDOWN_SECONDS', '60')),
    tradingEnabled:     !!process.env.POLYMARKET_PRIVATE_KEY,
  };
}
