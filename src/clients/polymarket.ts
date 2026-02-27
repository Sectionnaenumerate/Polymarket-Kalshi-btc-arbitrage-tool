import axios, { AxiosInstance } from 'axios';
import { ethers } from 'ethers';
import { MarketSide, PriceQuote } from '../types';

const DEFAULT_CLOB = 'https://clob.polymarket.com';

interface ClobPriceResponse  { price: string }
interface ClobOrderbookLevel { price: string; size: string }
interface ClobOrderbookResponse {
  bids: ClobOrderbookLevel[];
  asks: ClobOrderbookLevel[];
}

export class PolyClient {
  private http: AxiosInstance;
  private wallet?: ethers.Wallet;

  constructor(clobBase = DEFAULT_CLOB, privateKey?: string) {
    this.http = axios.create({ baseURL: clobBase, timeout: 10_000 });
    if (privateKey) {
      this.wallet = new ethers.Wallet(privateKey);
    }
  }

  get address(): string | undefined {
    return this.wallet?.address;
  }

  /** Get current mid-price for a token (returns cents 0–100). */
  async getPrice(tokenId: string, side: MarketSide): Promise<PriceQuote> {
    const { data } = await this.http.get<ClobPriceResponse>(
      `/price?token_id=${tokenId}&side=buy`
    );
    const priceCents = parseFloat(data.price) * 100;
    return {
      exchange: 'polymarket',
      side,
      priceCents,
      liquidityUsd: 0,
      fetchedAt: new Date(),
    };
  }

  /** Sum of top-5 bid levels in USD. */
  async getLiquidity(tokenId: string): Promise<number> {
    const { data } = await this.http.get<ClobOrderbookResponse>(`/book?token_id=${tokenId}`);
    return data.bids
      .slice(0, 5)
      .reduce((sum, lvl) => sum + parseFloat(lvl.price) * parseFloat(lvl.size), 0);
  }

  /**
   * Place a market buy order on the CLOB.
   * Signs the order using the EIP-712 / Polymarket scheme.
   */
  async placeBuy(tokenId: string, amountUsd: number): Promise<string> {
    if (!this.wallet) throw new Error('No private key configured');

    const quote = await this.getPrice(tokenId, 'YES');
    const priceFrac = quote.priceCents / 100;
    const size = priceFrac > 0 ? amountUsd / priceFrac : 0;
    const nonce = Date.now();

    // Build and sign order payload
    const orderData = {
      tokenId,
      side: 'BUY',
      price: priceFrac.toFixed(6),
      size: size.toFixed(6),
      timeInForce: 'FOK',
      nonce,
      maker: this.wallet.address,
    };

    const digest = ethers.keccak256(
      ethers.toUtf8Bytes(JSON.stringify(orderData))
    );
    const signature = await this.wallet.signMessage(ethers.getBytes(digest));

    const { data } = await this.http.post('/order', { ...orderData, signature });
    if (data.status === 'matched' || data.status === 'live') {
      return data.orderID as string;
    }
    throw new Error(`Order rejected: ${data.errorMsg ?? 'unknown'}`);
  }
}
