import 'dotenv/config';
import express from 'express';
import { loadConfig }    from './utils/config';
import { logger }        from './utils/logger';
import { PollerService } from './services/poller';
import { createRouter }  from './routes';

async function main(): Promise<void> {
  const cfg = loadConfig();

  logger.info('🚀 Polymarket-Kalshi BTC Arbitrage Tool (TypeScript layer)');
  logger.info(`   Kalshi ticker:      ${cfg.kalshiTicker}`);
  logger.info(`   Polymarket YES:     ${cfg.polymarketTokenYes}`);
  logger.info(`   Market start:       ${cfg.marketStartTime.toISOString()}`);
  logger.info(`   Start delay:        ${cfg.startDelayMins} min`);
  logger.info(`   Kalshi range:       ${cfg.kalshiMinCents}–${cfg.kalshiMaxCents}¢`);
  logger.info(`   Min spread:         ${cfg.minSpreadCents}¢`);
  logger.info(`   Trade size:         $${cfg.tradeUsd}`);
  logger.info(`   Trading:            ${cfg.tradingEnabled ? 'ENABLED' : 'DISABLED (signal-only)'}`);
  logger.info(`   Poll interval:      ${cfg.pollIntervalMs}ms`);

  const poller = new PollerService(cfg);
  poller.start();

  const app = express();
  app.use(express.json());
  app.use(createRouter(poller));

  app.listen(cfg.port, () => {
    logger.info(`🌐 HTTP API → http://localhost:${cfg.port}`);
    logger.info('   GET  /health');
    logger.info('   GET  /status');
    logger.info('   POST /poll/start');
    logger.info('   POST /poll/stop');
  });
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
