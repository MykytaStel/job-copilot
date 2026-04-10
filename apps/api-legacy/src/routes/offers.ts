import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { z } from 'zod';
import type { Offer, OfferInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { offers } from '../db/schema.js';

const inputSchema = z.object({
  jobId: z.string().min(1),
  salary: z.number().positive().optional(),
  currency: z.string().default('UAH'),
  equity: z.string().optional(),
  benefits: z.array(z.string()).default([]),
  deadline: z.string().optional(),
  notes: z.string().default(''),
});

function rowToOffer(row: typeof offers.$inferSelect): Offer {
  return {
    id: row.id,
    jobId: row.jobId,
    salary: row.salary ?? undefined,
    currency: row.currency,
    equity: row.equity ?? undefined,
    benefits: JSON.parse(row.benefits) as string[],
    deadline: row.deadline ?? undefined,
    notes: row.notes,
    createdAt: row.createdAt,
  };
}

export async function registerOfferRoutes(app: FastifyInstance): Promise<void> {
  app.get('/offers', async (): Promise<Offer[]> => {
    const rows = await db.select().from(offers);
    return rows.map(rowToOffer);
  });

  app.get<{ Params: { id: string } }>('/offers/:id', async (request, reply) => {
    const row = await db.select().from(offers).where(eq(offers.id, request.params.id)).get();
    if (!row) return reply.code(404).send({ error: 'Offer not found' });
    return rowToOffer(row);
  });

  app.post<{ Body: OfferInput }>('/offers', async (request, reply) => {
    const parsed = inputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid input', details: parsed.error.flatten() });
    }

    const d = parsed.data;
    const offer: Offer = {
      id: crypto.randomUUID(),
      jobId: d.jobId,
      salary: d.salary,
      currency: d.currency,
      equity: d.equity,
      benefits: d.benefits,
      deadline: d.deadline,
      notes: d.notes,
      createdAt: new Date().toISOString(),
    };

    await db.insert(offers).values({
      id: offer.id,
      jobId: offer.jobId,
      salary: offer.salary ?? null,
      currency: offer.currency,
      equity: offer.equity ?? null,
      benefits: JSON.stringify(offer.benefits),
      deadline: offer.deadline ?? null,
      notes: offer.notes,
      createdAt: offer.createdAt,
    });

    return reply.code(201).send(offer);
  });

  app.patch<{ Params: { id: string }; Body: Partial<OfferInput> }>(
    '/offers/:id',
    async (request, reply) => {
      const { id } = request.params;
      const row = await db.select().from(offers).where(eq(offers.id, id)).get();
      if (!row) return reply.code(404).send({ error: 'Offer not found' });

      const patch: Partial<typeof offers.$inferInsert> = {};
      const b = request.body;
      if (b.salary !== undefined) patch.salary = b.salary ?? null;
      if (b.currency) patch.currency = b.currency;
      if (b.equity !== undefined) patch.equity = b.equity ?? null;
      if (b.benefits) patch.benefits = JSON.stringify(b.benefits);
      if (b.deadline !== undefined) patch.deadline = b.deadline ?? null;
      if (b.notes !== undefined) patch.notes = b.notes;

      await db.update(offers).set(patch).where(eq(offers.id, id));
      return rowToOffer({ ...row, ...patch } as typeof offers.$inferSelect);
    }
  );

  app.delete<{ Params: { id: string } }>('/offers/:id', async (request, reply) => {
    await db.delete(offers).where(eq(offers.id, request.params.id));
    reply.code(204);
    return reply.send();
  });
}
