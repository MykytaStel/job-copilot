import type { FastifyInstance } from 'fastify';
import { eq, desc } from 'drizzle-orm';
import type { MatchResult } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { jobs, resumes, matchResults } from '../db/schema.js';
import { callClaudeJson } from '../lib/claude.js';
import { fitScoringPrompt } from '../lib/prompts.js';

interface FitResponse {
  score: number;
  matchedSkills: string[];
  missingSkills: string[];
  notes: string;
}

export async function registerMatchRoutes(app: FastifyInstance): Promise<void> {
  // Run AI fit scoring for a job against the current active resume
  app.post<{ Params: { id: string } }>('/jobs/:id/match', async (request, reply) => {
    const job = await db.select().from(jobs).where(eq(jobs.id, request.params.id)).get();
    if (!job) return reply.code(404).send({ error: 'Job not found' });

    const resume = await db.select().from(resumes).where(eq(resumes.isActive, true)).get();
    if (!resume) return reply.code(422).send({ error: 'No resume uploaded yet' });

    let fit: FitResponse;
    try {
      fit = await callClaudeJson<FitResponse>(
        fitScoringPrompt(job.description, resume.rawText),
        1024,
      );
    } catch (err) {
      return reply.code(503).send({
        error: `AI analysis failed: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    }

    // Clamp score to 0-100 in case LLM goes out of range
    const score = Math.max(0, Math.min(100, Math.round(fit.score)));

    const now = new Date().toISOString();
    const id = crypto.randomUUID();

    await db.insert(matchResults).values({
      id,
      jobId: job.id,
      resumeId: resume.id,
      score,
      matchedSkills: JSON.stringify(fit.matchedSkills ?? []),
      missingSkills: JSON.stringify(fit.missingSkills ?? []),
      notes: fit.notes ?? '',
      createdAt: now,
    });

    const result: MatchResult = {
      id,
      jobId: job.id,
      resumeId: resume.id,
      score,
      matchedSkills: fit.matchedSkills ?? [],
      missingSkills: fit.missingSkills ?? [],
      notes: fit.notes ?? '',
      createdAt: now,
    };

    return reply.code(201).send(result);
  });

  // Get most recent match result for a job
  app.get<{ Params: { id: string } }>('/jobs/:id/match', async (request, reply) => {
    const row = await db
      .select()
      .from(matchResults)
      .where(eq(matchResults.jobId, request.params.id))
      .orderBy(desc(matchResults.createdAt))
      .get();

    if (!row) {
      return reply.code(404).send({ error: 'No match result yet. POST /jobs/:id/match first.' });
    }

    return rowToMatch(row);
  });
}

function rowToMatch(row: typeof matchResults.$inferSelect): MatchResult {
  return {
    id: row.id,
    jobId: row.jobId,
    resumeId: row.resumeId,
    score: row.score,
    matchedSkills: JSON.parse(row.matchedSkills) as string[],
    missingSkills: JSON.parse(row.missingSkills) as string[],
    notes: row.notes,
    createdAt: row.createdAt,
  };
}
