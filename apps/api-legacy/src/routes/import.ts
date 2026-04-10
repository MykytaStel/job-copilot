import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { z } from 'zod';
import type { ImportBatchResponse } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { jobs } from '../db/schema.js';
import { fetchJobFromUrl } from '../lib/scraper.js';

const batchSchema = z.object({
  urls: z.array(z.string().url()).min(1).max(20),
});

export async function registerImportRoutes(app: FastifyInstance): Promise<void> {
  /**
   * POST /import/batch
   * Accepts up to 20 job URLs (Djinni, Work.ua, Robota.ua, or any site).
   * Skips duplicates (matched by URL). Returns per-URL result.
   */
  app.post<{ Body: { urls: string[] } }>('/import/batch', async (request, reply) => {
    const parsed = batchSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid input', details: parsed.error.flatten() });
    }

    const response: ImportBatchResponse = { results: [] };

    for (const url of parsed.data.urls) {
      const existing = await db.select().from(jobs).where(eq(jobs.url, url)).get();
      if (existing) {
        response.results.push({ url, status: 'duplicate' });
        continue;
      }

      try {
        const scraped = await fetchJobFromUrl(url);
        const job = {
          id: crypto.randomUUID(),
          source: 'url' as const,
          url,
          title: scraped.title || 'Untitled',
          company: scraped.company || 'Unknown',
          description: scraped.description,
          notes: '',
          createdAt: new Date().toISOString(),
        };
        await db.insert(jobs).values(job);
        response.results.push({ url, status: 'imported', job });
      } catch (err) {
        response.results.push({ url, status: 'error', error: String(err) });
      }
    }

    return response;
  });
}
