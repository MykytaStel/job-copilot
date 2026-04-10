import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { z } from 'zod';
import type { CoverLetter, CoverLetterInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { coverLetters } from '../db/schema.js';

const TONES = ['formal', 'casual', 'enthusiastic'] as const;

const inputSchema = z.object({
  jobId: z.string().min(1),
  tone: z.enum(TONES).default('formal'),
  content: z.string().optional(),
});

function rowToLetter(row: typeof coverLetters.$inferSelect): CoverLetter {
  return {
    id: row.id,
    jobId: row.jobId,
    content: row.content,
    tone: row.tone as CoverLetter['tone'],
    createdAt: row.createdAt,
  };
}

export async function registerCoverLetterRoutes(app: FastifyInstance): Promise<void> {
  app.get<{ Querystring: { jobId?: string } }>('/cover-letters', async (request) => {
    const { jobId } = request.query;
    const rows = jobId
      ? await db.select().from(coverLetters).where(eq(coverLetters.jobId, jobId))
      : await db.select().from(coverLetters);
    return rows.map(rowToLetter);
  });

  app.post<{ Body: CoverLetterInput }>('/cover-letters', async (request, reply) => {
    const parsed = inputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid input', details: parsed.error.flatten() });
    }

    const { jobId, tone, content } = parsed.data;

    // AI placeholder: when AI is enabled, generate from job description + active resume
    const finalContent =
      content ??
      `[Cover letter for ${tone} tone — click Edit to write your letter, or enable AI generation in settings.]`;

    const letter: CoverLetter = {
      id: crypto.randomUUID(),
      jobId,
      content: finalContent,
      tone,
      createdAt: new Date().toISOString(),
    };

    await db.insert(coverLetters).values({
      id: letter.id,
      jobId: letter.jobId,
      content: letter.content,
      tone: letter.tone,
      createdAt: letter.createdAt,
    });

    return reply.code(201).send(letter);
  });

  app.patch<{ Params: { id: string }; Body: { content?: string; tone?: string } }>(
    '/cover-letters/:id',
    async (request, reply) => {
      const { id } = request.params;
      const row = await db.select().from(coverLetters).where(eq(coverLetters.id, id)).get();
      if (!row) return reply.code(404).send({ error: 'Cover letter not found' });

      const patch: Partial<typeof coverLetters.$inferInsert> = {};
      if (request.body.content !== undefined) patch.content = request.body.content;
      if (request.body.tone !== undefined) patch.tone = request.body.tone;

      await db.update(coverLetters).set(patch).where(eq(coverLetters.id, id));
      return rowToLetter({ ...row, ...patch } as typeof coverLetters.$inferSelect);
    }
  );

  app.delete<{ Params: { id: string } }>('/cover-letters/:id', async (request, reply) => {
    await db.delete(coverLetters).where(eq(coverLetters.id, request.params.id));
    reply.code(204);
    return reply.send();
  });
}
