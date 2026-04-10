import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq, desc } from 'drizzle-orm';
import type { JobPosting, JobPostingInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { jobs, applications, applicationNotes, matchResults, alerts } from '../db/schema.js';
import { fetchJobFromUrl } from '../lib/scraper.js';
import { sendTelegramMessage } from '../lib/telegram.js';

const jobInputSchema = z.object({
  source: z.enum(['manual', 'url', 'rss']),
  url: z.string().url().optional(),
  rawText: z.string().min(10).optional(),
  title: z.string().optional(),
  company: z.string().optional(),
});

export async function registerJobRoutes(app: FastifyInstance): Promise<void> {
  app.get('/jobs', async (): Promise<JobPosting[]> => {
    const rows = await db.select().from(jobs).orderBy(desc(jobs.createdAt));
    return rows.map(rowToJob);
  });

  app.get<{ Params: { id: string } }>('/jobs/:id', async (request, reply) => {
    const row = await db.select().from(jobs).where(eq(jobs.id, request.params.id)).get();
    if (!row) return reply.code(404).send({ error: 'Job not found' });
    return rowToJob(row);
  });

  // Fetch and parse a job posting from a URL (djinni.co, work.ua, robota.ua, or generic)
  app.post<{ Body: { url: string } }>('/jobs/fetch-url', async (request, reply) => {
    const parsed = z.object({ url: z.string().url() }).safeParse(request.body);
    if (!parsed.success) return reply.code(400).send({ error: 'Invalid URL' });

    try {
      const result = await fetchJobFromUrl(parsed.data.url);
      return result;
    } catch (err) {
      return reply.code(400).send({
        error: `Could not fetch URL: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    }
  });

  app.post<{ Body: JobPostingInput }>('/jobs/intake', async (request, reply) => {
    const parsed = jobInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({
        error: 'Invalid job intake payload',
        details: parsed.error.flatten(),
      });
    }

    // Dedup by URL
    if (parsed.data.url) {
      const existing = await db.select().from(jobs).where(eq(jobs.url, parsed.data.url)).get();
      if (existing) {
        return reply.code(409).send({ error: `Job already saved:${existing.id}`, id: existing.id });
      }
    }

    const job: JobPosting = {
      id: crypto.randomUUID(),
      source: parsed.data.source,
      url: parsed.data.url,
      title: parsed.data.title ?? 'Untitled job',
      company: parsed.data.company ?? 'Unknown company',
      description: parsed.data.rawText ?? 'No description provided yet.',
      notes: '',
      createdAt: new Date().toISOString(),
    };

    await db.insert(jobs).values({
      id: job.id,
      source: job.source,
      url: job.url ?? null,
      title: job.title,
      company: job.company,
      description: job.description,
      notes: '',
      createdAt: job.createdAt,
    });

    void fireAlertsForJob(job);
    return reply.code(201).send(job);
  });

  app.patch<{ Params: { id: string }; Body: { note: string } }>(
    '/jobs/:id/note',
    async (request, reply) => {
      const { id } = request.params;
      const { note } = request.body;
      if (typeof note !== 'string') return reply.code(400).send({ error: 'note must be a string' });
      const row = await db.select().from(jobs).where(eq(jobs.id, id)).get();
      if (!row) return reply.code(404).send({ error: 'Job not found' });
      await db.update(jobs).set({ notes: note }).where(eq(jobs.id, id));
      return rowToJob({ ...row, notes: note });
    },
  );

  app.delete<{ Params: { id: string } }>('/jobs/:id', async (request, reply) => {
    const { id } = request.params;
    const job = await db.select().from(jobs).where(eq(jobs.id, id)).get();
    if (!job) return reply.code(404).send({ error: 'Job not found' });

    // Cascade: delete notes → applications → match results → job
    const appRows = await db.select().from(applications).where(eq(applications.jobId, id));
    for (const app of appRows) {
      await db.delete(applicationNotes).where(eq(applicationNotes.applicationId, app.id));
    }
    await db.delete(applications).where(eq(applications.jobId, id));
    await db.delete(matchResults).where(eq(matchResults.jobId, id));
    await db.delete(jobs).where(eq(jobs.id, id));

    reply.code(204);
    return reply.send();
  });
}

async function fireAlertsForJob(job: JobPosting): Promise<void> {
  const activeAlerts = await db.select().from(alerts).where(eq(alerts.active, true));
  const searchText = `${job.title} ${job.description}`.toLowerCase();

  for (const alert of activeAlerts) {
    const keywords = JSON.parse(alert.keywords) as string[];
    const matched = keywords.filter((kw) => searchText.includes(kw.toLowerCase()));
    if (matched.length === 0) continue;

    const message =
      `🔔 <b>Нова вакансія!</b>\n` +
      `<b>${job.title}</b> — ${job.company}\n` +
      `Збіг за: ${matched.join(', ')}\n` +
      (job.url ? `<a href="${job.url}">${job.url}</a>` : '');

    sendTelegramMessage(alert.telegramChatId, message).catch(() => null);
  }
}

function rowToJob(row: typeof jobs.$inferSelect): JobPosting {
  return {
    id: row.id,
    source: row.source as JobPosting['source'],
    url: row.url ?? undefined,
    title: row.title,
    company: row.company,
    description: row.description,
    notes: row.notes,
    createdAt: row.createdAt,
  };
}
