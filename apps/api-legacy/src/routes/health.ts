import type { FastifyInstance } from 'fastify';
import type { HealthResponse } from '@job-copilot/shared';

export async function registerHealthRoutes(app: FastifyInstance): Promise<void> {
  app.get<{ Reply: HealthResponse }>('/health', async () => {
    return {
      status: 'ok',
      service: 'job-copilot-api',
      timestamp: new Date().toISOString(),
    };
  });
}
