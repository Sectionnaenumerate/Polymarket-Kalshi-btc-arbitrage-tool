import { Router, Request, Response } from 'express';
import { PollerService } from '../services/poller';

export function createRouter(poller: PollerService): Router {
  const router = Router();

  /** GET /health */
  router.get('/health', (_req: Request, res: Response) => {
    res.json({ status: 'ok', service: 'polymarket-kalshi-btc-arbitrage-tool' });
  });

  /** GET /status */
  router.get('/status', (_req: Request, res: Response) => {
    res.json(poller.status);
  });

  /** POST /poll/start */
  router.post('/poll/start', (_req: Request, res: Response) => {
    poller.start();
    res.json({ pollingActive: true });
  });

  /** POST /poll/stop */
  router.post('/poll/stop', (_req: Request, res: Response) => {
    poller.stop();
    res.json({ pollingActive: false });
  });

  return router;
}
