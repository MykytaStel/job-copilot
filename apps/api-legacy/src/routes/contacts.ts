import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq } from 'drizzle-orm';
import type { Contact, ContactInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { contacts } from '../db/schema.js';

const contactInputSchema = z.object({
  name: z.string().min(1),
  email: z.string().email().optional(),
  phone: z.string().optional(),
  linkedinUrl: z.string().url().optional(),
  company: z.string().optional(),
  role: z.string().optional(),
});

const contactPatchSchema = contactInputSchema.partial();

export async function registerContactRoutes(app: FastifyInstance): Promise<void> {
  app.get('/contacts', async (): Promise<Contact[]> => {
    const rows = await db.select().from(contacts).orderBy(contacts.name);
    return rows.map(rowToContact);
  });

  app.post<{ Body: ContactInput }>('/contacts', async (request, reply) => {
    const parsed = contactInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid contact', details: parsed.error.flatten() });
    }

    const now = new Date().toISOString();
    const id = crypto.randomUUID();

    await db.insert(contacts).values({
      id,
      name: parsed.data.name,
      email: parsed.data.email ?? null,
      phone: parsed.data.phone ?? null,
      linkedinUrl: parsed.data.linkedinUrl ?? null,
      company: parsed.data.company ?? null,
      role: parsed.data.role ?? null,
      createdAt: now,
    });

    const row = await db.select().from(contacts).where(eq(contacts.id, id)).get();
    return reply.code(201).send(rowToContact(row!));
  });

  app.patch<{ Params: { id: string }; Body: Partial<ContactInput> }>(
    '/contacts/:id',
    async (request, reply) => {
      const parsed = contactPatchSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid patch', details: parsed.error.flatten() });
      }

      const target = await db.select().from(contacts).where(eq(contacts.id, request.params.id)).get();
      if (!target) return reply.code(404).send({ error: 'Contact not found' });

      const updates: Partial<typeof contacts.$inferInsert> = {};
      if (parsed.data.name !== undefined) updates.name = parsed.data.name;
      if ('email' in parsed.data) updates.email = parsed.data.email ?? null;
      if ('phone' in parsed.data) updates.phone = parsed.data.phone ?? null;
      if ('linkedinUrl' in parsed.data) updates.linkedinUrl = parsed.data.linkedinUrl ?? null;
      if ('company' in parsed.data) updates.company = parsed.data.company ?? null;
      if ('role' in parsed.data) updates.role = parsed.data.role ?? null;

      await db.update(contacts).set(updates).where(eq(contacts.id, request.params.id));

      const updated = await db.select().from(contacts).where(eq(contacts.id, request.params.id)).get();
      return rowToContact(updated!);
    },
  );

  app.delete<{ Params: { id: string } }>('/contacts/:id', async (request, reply) => {
    const target = await db.select().from(contacts).where(eq(contacts.id, request.params.id)).get();
    if (!target) return reply.code(404).send({ error: 'Contact not found' });

    await db.delete(contacts).where(eq(contacts.id, request.params.id));
    reply.code(204);
    return reply.send();
  });
}

function rowToContact(row: typeof contacts.$inferSelect): Contact {
  return {
    id: row.id,
    name: row.name,
    email: row.email ?? undefined,
    phone: row.phone ?? undefined,
    linkedinUrl: row.linkedinUrl ?? undefined,
    company: row.company ?? undefined,
    role: row.role ?? undefined,
    createdAt: row.createdAt,
  };
}
