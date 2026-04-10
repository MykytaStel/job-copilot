import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { z } from 'zod';
import type { InterviewQA, InterviewQAInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { interviewQA } from '../db/schema.js';

const CATEGORIES = ['behavioral', 'technical', 'situational', 'company'] as const;

const inputSchema = z.object({
  jobId: z.string().min(1),
  question: z.string().min(1),
  answer: z.string().default(''),
  category: z.enum(CATEGORIES).default('behavioral'),
});

function rowToQA(row: typeof interviewQA.$inferSelect): InterviewQA {
  return {
    id: row.id,
    jobId: row.jobId,
    question: row.question,
    answer: row.answer,
    category: row.category as InterviewQA['category'],
    createdAt: row.createdAt,
  };
}

export async function registerInterviewQARoutes(app: FastifyInstance): Promise<void> {
  app.get<{ Querystring: { jobId?: string } }>('/interview-qa', async (request) => {
    const { jobId } = request.query;
    const rows = jobId
      ? await db.select().from(interviewQA).where(eq(interviewQA.jobId, jobId))
      : await db.select().from(interviewQA);
    return rows.map(rowToQA);
  });

  app.post<{ Body: InterviewQAInput }>('/interview-qa', async (request, reply) => {
    const parsed = inputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid input', details: parsed.error.flatten() });
    }

    const qa: InterviewQA = {
      id: crypto.randomUUID(),
      jobId: parsed.data.jobId,
      question: parsed.data.question,
      answer: parsed.data.answer,
      category: parsed.data.category,
      createdAt: new Date().toISOString(),
    };

    await db.insert(interviewQA).values({
      id: qa.id,
      jobId: qa.jobId,
      question: qa.question,
      answer: qa.answer,
      category: qa.category,
      createdAt: qa.createdAt,
    });

    return reply.code(201).send(qa);
  });

  app.patch<{ Params: { id: string }; Body: { question?: string; answer?: string } }>(
    '/interview-qa/:id',
    async (request, reply) => {
      const { id } = request.params;
      const row = await db.select().from(interviewQA).where(eq(interviewQA.id, id)).get();
      if (!row) return reply.code(404).send({ error: 'Q&A not found' });

      const patch: Partial<typeof interviewQA.$inferInsert> = {};
      if (request.body.question !== undefined) patch.question = request.body.question;
      if (request.body.answer !== undefined) patch.answer = request.body.answer;

      await db.update(interviewQA).set(patch).where(eq(interviewQA.id, id));
      return rowToQA({ ...row, ...patch } as typeof interviewQA.$inferSelect);
    }
  );

  app.delete<{ Params: { id: string } }>('/interview-qa/:id', async (request, reply) => {
    await db.delete(interviewQA).where(eq(interviewQA.id, request.params.id));
    reply.code(204);
    return reply.send();
  });

  // AI placeholder: returns 503 until AI is enabled
  app.post<{ Body: { jobId: string } }>('/interview-qa/generate', async (_request, reply) => {
    return reply.code(503).send({
      error: 'AI generation not yet enabled. Add ANTHROPIC_API_KEY to enable.',
    });
  });
}
