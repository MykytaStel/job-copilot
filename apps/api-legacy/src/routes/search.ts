import type { FastifyInstance } from 'fastify';
import { like, or } from 'drizzle-orm';
import type { SearchResults } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { jobs, contacts } from '../db/schema.js';

export async function registerSearchRoutes(app: FastifyInstance): Promise<void> {
  app.get<{ Querystring: { q?: string } }>('/search', async (request): Promise<SearchResults> => {
    const q = (request.query.q ?? '').trim();

    if (!q || q.length < 2) {
      return { jobs: [], contacts: [] };
    }

    const pattern = `%${q}%`;

    const jobRows = await db
      .select()
      .from(jobs)
      .where(
        or(
          like(jobs.title, pattern),
          like(jobs.company, pattern),
          like(jobs.description, pattern),
          like(jobs.notes, pattern),
        ),
      )
      .limit(8);

    const contactRows = await db
      .select()
      .from(contacts)
      .where(
        or(
          like(contacts.name, pattern),
          like(contacts.email, pattern),
          like(contacts.company, pattern),
          like(contacts.role, pattern),
        ),
      )
      .limit(5);

    return {
      jobs: jobRows.map((r) => ({
        id: r.id,
        source: r.source as 'manual' | 'url' | 'rss',
        url: r.url ?? undefined,
        title: r.title,
        company: r.company,
        description: r.description,
        notes: r.notes,
        createdAt: r.createdAt,
      })),
      contacts: contactRows.map((r) => ({
        id: r.id,
        name: r.name,
        email: r.email ?? undefined,
        phone: r.phone ?? undefined,
        linkedinUrl: r.linkedinUrl ?? undefined,
        company: r.company ?? undefined,
        role: r.role ?? undefined,
        createdAt: r.createdAt,
      })),
    };
  });
}
