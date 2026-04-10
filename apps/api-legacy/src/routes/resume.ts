import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq, desc, sql } from 'drizzle-orm';
import { createRequire } from 'module';
const require = createRequire(import.meta.url);
// pdf-parse@1.x is CJS-only; createRequire bridges the ESM/CJS gap
const pdfParse = require('pdf-parse') as (buf: Buffer) => Promise<{ text: string }>;
import type { ResumeVersion, ResumeUploadInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { resumes } from '../db/schema.js';

const resumeInputSchema = z.object({
  filename: z.string().min(1),
  rawText: z.string().min(10),
});

export async function registerResumeRoutes(app: FastifyInstance): Promise<void> {
  app.get('/resumes', async (): Promise<ResumeVersion[]> => {
    const rows = await db.select().from(resumes).orderBy(desc(resumes.version));
    return rows.map(rowToResume);
  });

  app.get('/resumes/active', async (_req, reply) => {
    const row = await db
      .select()
      .from(resumes)
      .where(eq(resumes.isActive, true))
      .get();

    if (!row) return reply.code(404).send({ error: 'No active resume' });
    return rowToResume(row);
  });

  app.post<{ Body: ResumeUploadInput }>('/resume/upload', async (request, reply) => {
    const parsed = resumeInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid resume', details: parsed.error.flatten() });
    }

    // Determine next version number
    const countRow = await db
      .select({ max: sql<number>`MAX(version)` })
      .from(resumes)
      .get();
    const nextVersion = (countRow?.max ?? 0) + 1;

    const now = new Date().toISOString();
    const id = crypto.randomUUID();

    // Deactivate all previous resumes then insert new active one
    await db.update(resumes).set({ isActive: false });

    await db.insert(resumes).values({
      id,
      version: nextVersion,
      filename: parsed.data.filename,
      rawText: parsed.data.rawText,
      isActive: true,
      uploadedAt: now,
    });

    const resume: ResumeVersion = {
      id,
      version: nextVersion,
      filename: parsed.data.filename,
      rawText: parsed.data.rawText,
      isActive: true,
      uploadedAt: now,
    };

    return reply.code(201).send(resume);
  });

  // Multipart file upload — supports .pdf, .txt, .md
  app.post('/resume/upload-file', async (request, reply) => {
    const data = await request.file();
    if (!data) return reply.code(400).send({ error: 'No file provided' });

    const buffer = await data.toBuffer();
    const filename = data.filename || 'resume';

    let rawText: string;
    if (filename.toLowerCase().endsWith('.pdf')) {
      const result = await pdfParse(buffer);
      rawText = result.text.trim();
    } else {
      rawText = buffer.toString('utf-8').trim();
    }

    if (rawText.length < 10) {
      return reply.code(400).send({ error: 'Could not extract enough text from file' });
    }

    const countRow = await db.select({ max: sql<number>`MAX(version)` }).from(resumes).get();
    const nextVersion = (countRow?.max ?? 0) + 1;
    const now = new Date().toISOString();
    const id = crypto.randomUUID();

    await db.update(resumes).set({ isActive: false });
    await db.insert(resumes).values({ id, version: nextVersion, filename, rawText, isActive: true, uploadedAt: now });

    const resume: ResumeVersion = { id, version: nextVersion, filename, rawText, isActive: true, uploadedAt: now };
    return reply.code(201).send(resume);
  });

  app.post<{ Params: { id: string } }>('/resumes/:id/activate', async (request, reply) => {
    const target = await db
      .select()
      .from(resumes)
      .where(eq(resumes.id, request.params.id))
      .get();

    if (!target) return reply.code(404).send({ error: 'Resume not found' });

    await db.update(resumes).set({ isActive: false });
    await db.update(resumes).set({ isActive: true }).where(eq(resumes.id, request.params.id));

    return rowToResume({ ...target, isActive: true });
  });
}

function rowToResume(row: typeof resumes.$inferSelect): ResumeVersion {
  return {
    id: row.id,
    version: row.version,
    filename: row.filename,
    rawText: row.rawText,
    isActive: Boolean(row.isActive),
    uploadedAt: row.uploadedAt,
  };
}
