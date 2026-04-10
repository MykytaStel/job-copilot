import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq, desc, and } from 'drizzle-orm';
import type { Task, TaskInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { tasks, applications } from '../db/schema.js';

const taskInputSchema = z.object({
  title: z.string().min(1),
  remindAt: z.string().optional(),
});

const taskPatchSchema = z.object({
  title: z.string().min(1).optional(),
  remindAt: z.string().nullable().optional(),
  done: z.boolean().optional(),
});

export async function registerTaskRoutes(app: FastifyInstance): Promise<void> {
  // Tasks due in the next 48 h (across all applications) — used by Dashboard
  app.get('/tasks/due', async (): Promise<Task[]> => {
    const now = new Date();
    const cutoff = new Date(now.getTime() + 48 * 60 * 60 * 1000).toISOString();

    const rows = await db
      .select()
      .from(tasks)
      .where(and(eq(tasks.done, false)));

    return rows
      .filter((r) => r.remindAt && r.remindAt <= cutoff)
      .map(rowToTask);
  });

  app.get<{ Params: { id: string } }>(
    '/applications/:id/tasks',
    async (request, reply) => {
      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();
      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const rows = await db
        .select()
        .from(tasks)
        .where(eq(tasks.applicationId, request.params.id))
        .orderBy(desc(tasks.createdAt));

      return rows.map(rowToTask);
    },
  );

  app.post<{ Params: { id: string }; Body: TaskInput }>(
    '/applications/:id/tasks',
    async (request, reply) => {
      const parsed = taskInputSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid task', details: parsed.error.flatten() });
      }

      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();
      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const now = new Date().toISOString();
      const id = crypto.randomUUID();

      await db.insert(tasks).values({
        id,
        applicationId: request.params.id,
        title: parsed.data.title,
        remindAt: parsed.data.remindAt ?? null,
        done: false,
        createdAt: now,
      });

      return reply.code(201).send(rowToTask({
        id,
        applicationId: request.params.id,
        title: parsed.data.title,
        remindAt: parsed.data.remindAt ?? null,
        done: false,
        createdAt: now,
      }));
    },
  );

  app.patch<{ Params: { id: string }; Body: { title?: string; remindAt?: string | null; done?: boolean } }>(
    '/tasks/:id',
    async (request, reply) => {
      const parsed = taskPatchSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid patch', details: parsed.error.flatten() });
      }

      const target = await db.select().from(tasks).where(eq(tasks.id, request.params.id)).get();
      if (!target) return reply.code(404).send({ error: 'Task not found' });

      const updates: Partial<typeof tasks.$inferInsert> = {};
      if (parsed.data.title !== undefined) updates.title = parsed.data.title;
      if ('remindAt' in parsed.data) updates.remindAt = parsed.data.remindAt ?? null;
      if (parsed.data.done !== undefined) updates.done = parsed.data.done;

      await db.update(tasks).set(updates).where(eq(tasks.id, request.params.id));

      const updated = await db.select().from(tasks).where(eq(tasks.id, request.params.id)).get();
      return rowToTask(updated!);
    },
  );

  app.delete<{ Params: { id: string } }>('/tasks/:id', async (request, reply) => {
    const target = await db.select().from(tasks).where(eq(tasks.id, request.params.id)).get();
    if (!target) return reply.code(404).send({ error: 'Task not found' });

    await db.delete(tasks).where(eq(tasks.id, request.params.id));
    reply.code(204);
    return reply.send();
  });
}

function rowToTask(row: typeof tasks.$inferSelect): Task {
  return {
    id: row.id,
    applicationId: row.applicationId,
    title: row.title,
    remindAt: row.remindAt ?? undefined,
    done: Boolean(row.done),
    createdAt: row.createdAt,
  };
}
