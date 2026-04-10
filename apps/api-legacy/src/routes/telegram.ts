import type { FastifyInstance } from 'fastify';
import { handleWebhook, setupWebhook } from '../lib/bot.js';

export async function registerTelegramRoutes(app: FastifyInstance): Promise<void> {
  /**
   * POST /telegram/webhook
   * Telegram sends updates here. Register this URL via /telegram/setup-webhook.
   */
  app.post('/telegram/webhook', async (request, reply) => {
    await handleWebhook(request.body);
    reply.code(200);
    return reply.send({ ok: true });
  });

  /**
   * POST /telegram/setup-webhook
   * Body: { url: "https://your-domain.com/telegram/webhook" }
   * Run once after deploying to a public HTTPS server.
   */
  app.post<{ Body: { url: string } }>('/telegram/setup-webhook', async (request, reply) => {
    const { url } = request.body ?? {};
    if (!url) return reply.code(400).send({ error: 'url is required' });
    await setupWebhook(url);
    return { ok: true, webhookUrl: url };
  });
}
