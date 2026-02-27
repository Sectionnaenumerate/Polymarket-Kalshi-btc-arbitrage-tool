import axios, { AxiosInstance } from 'axios';
import { KalshiStatus, PriceQuote } from '../types';

const DEFAULT_BASE = 'https://api.elections.kalshi.com/trade-api/v2';

interface KalshiMarketResponse {
  market: {
    ticker: string;
    status: string;
    yes_bid?: number;
    yes_ask?: number;
    volume?: number;
  };
}

interface KalshiOrderbookResponse {
  orderbook: {
    yes: [number, number][];  // [price_cents, qty]
    no:  [number, number][];
  };
}

export class KalshiClient {
  private http: AxiosInstance;

  constructor(base = DEFAULT_BASE, token?: string) {
    this.http = axios.create({
      baseURL: base,
      timeout: 10_000,
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
  }

  /** Fetch YES mid-price and market status for a BTC 15-min ticker. */
  async getBtcPrice(ticker: string): Promise<{ quote: PriceQuote; status: KalshiStatus }> {
    const { data } = await this.http.get<KalshiMarketResponse>(`/markets/${ticker}`);
    const m = data.market;

    const status: KalshiStatus = (['open', 'closed', 'settled'] as const).includes(
      m.status as KalshiStatus
    )
      ? (m.status as KalshiStatus)
      : 'unknown';

    const priceCents =
      m.yes_bid != null && m.yes_ask != null
        ? (m.yes_bid + m.yes_ask) / 2
        : (m.yes_bid ?? 0);

    return {
      quote: {
        exchange: 'kalshi',
        side: 'YES',
        priceCents,
        liquidityUsd: m.volume ?? 0,
        fetchedAt: new Date(),
      },
      status,
    };
  }

  /** Sum of top-3 YES bid levels in USD. */
  async getYesLiquidity(ticker: string): Promise<number> {
    const { data } = await this.http.get<KalshiOrderbookResponse>(`/markets/${ticker}/orderbook`);
    return data.orderbook.yes
      .slice(0, 3)
      .reduce((sum, [px, qty]) => sum + (px / 100) * qty, 0);
  }
}
