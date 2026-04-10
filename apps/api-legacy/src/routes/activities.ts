import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq, desc } from 'drizzle-orm';
import type { Activity, ActivityInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { activities, applications } from '../db/schema.js';

const activityInputSchema = z.object({
  type: z.enum(['email', 'call', 'interview', 'follow_up', 'note', 'other']),
  description: z.string().min(1),
  happenedAt: z.string(),
});

export async function registerActivityRoutes(app: FastifyInstance): Promise<void> {
  app.get<{ Params: { id: string } }>(
    '/applications/:id/activities',
    async (request, reply) => {
      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();
      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const rows = await db
        .select()
        .from(activities)
        .where(eq(activities.applicationId, request.params.id))
        .orderBy(desc(activities.happenedAt));

      return rows.map(rowToActivity);
    },
  );

  app.post<{ Params: { id: string }; Body: ActivityInput }>(
    '/applications/:id/activities',
    async (request, reply) => {
      const parsed = activityInputSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid activity', details: parsed.error.flatten() });
      }

      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();
      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const now = new Date().toISOString();
      const id = crypto.randomUUID();

      await db.insert(activities).values({
        id,
        applicationId: request.params.id,
        type: parsed.data.type,
        description: parsed.data.description,
        happenedAt: parsed.data.happenedAt,
        createdAt: now,
      });

      const activity: Activity = {
        id,
        applicationId: request.params.id,
        type: parsed.data.type,
        description: parsed.data.description,
        happenedAt: parsed.data.happenedAt,
        createdAt: now,
      };

      return reply.code(201).send(activity);
    },
  );

  app.delete<{ Params: { id: string } }>('/activities/:id', async (request, reply) => {
    const target = await db
      .select()
      .from(activities)
      .where(eq(activities.id, request.params.id))
      .get();
    if (!target) return reply.code(404).send({ error: 'Activity not found' });

    await db.delete(activities).where(eq(activities.id, request.params.id));
    reply.code(204);
    return reply.send();
  });
}

function rowToActivity(row: typeof activities.$inferSelect): Activity {
  return {
    id: row.id,
    applicationId: row.applicationId,
    type: row.type as Activity['type'],
    description: row.description,
    happenedAt: row.happenedAt,
    createdAt: row.createdAt,
  };
}
