import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { z } from 'zod';
import type { JobAlert, JobAlertInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { alerts } from '../db/schema.js';

const alertInputSchema = z.object({
  keywords: z.array(z.string().min(1)).min(1),
  telegramChatId: z.string().min(1),
});

function rowToAlert(row: typeof alerts.$inferSelect): JobAlert {
  return {
    id: row.id,
    keywords: JSON.parse(row.keywords) as string[],
    telegramChatId: row.telegramChatId,
    active: row.active,
    createdAt: row.createdAt,
  };
}

export async function registerAlertRoutes(app: FastifyInstance): Promise<void> {
  app.get('/alerts', async (): Promise<JobAlert[]> => {
    const rows = await db.select().from(alerts);
    return rows.map(rowToAlert);
  });

  app.post<{ Body: JobAlertInput }>('/alerts', async (request, reply) => {
    const parsed = alertInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid alert payload', details: parsed.error.flatten() });
    }

    const alert: JobAlert = {
      id: crypto.randomUUID(),
      keywords: parsed.data.keywords,
      telegramChatId: parsed.data.telegramChatId,
      active: true,
      createdAt: new Date().toISOString(),
    };

    await db.insert(alerts).values({
      id: alert.id,
      keywords: JSON.stringify(alert.keywords),
      telegramChatId: alert.telegramChatId,
      active: true,
      createdAt: alert.createdAt,
    });

    return reply.code(201).send(alert);
  });

  app.patch<{ Params: { id: string }; Body: { active: boolean } }>(
    '/alerts/:id',
    async (request, reply) => {
      const { id } = request.params;
      const { active } = request.body;
      const row = await db.select().from(alerts).where(eq(alerts.id, id)).get();
      if (!row) return reply.code(404).send({ error: 'Alert not found' });
      await db.update(alerts).set({ active }).where(eq(alerts.id, id));
      return rowToAlert({ ...row, active });
    }
  );

  app.delete<{ Params: { id: string } }>('/alerts/:id', async (request, reply) => {
    const { id } = request.params;
    await db.delete(alerts).where(eq(alerts.id, id));
    reply.code(204);
    return reply.send();
  });
}
