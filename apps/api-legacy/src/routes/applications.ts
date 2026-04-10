import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq, desc } from 'drizzle-orm';
import type {
  Activity,
  Application,
  ApplicationContact,
  ApplicationDetail,
  ApplicationInput,
  ApplicationNote,
  Contact,
  ContactRelationship,
  JobPosting,
  ResumeVersion,
  Task,
} from '@job-copilot/shared';
import { db } from '../db/index.js';
import {
  applications,
  applicationNotes,
  applicationContacts,
  contacts,
  activities,
  tasks,
  jobs,
  resumes,
} from '../db/schema.js';

const applicationInputSchema = z.object({
  jobId: z.string().uuid(),
  status: z.enum(['saved', 'applied', 'interview', 'offer', 'rejected']),
  appliedAt: z.string().optional(),
});

const statusPatchSchema = z.object({
  status: z.enum(['saved', 'applied', 'interview', 'offer', 'rejected']).optional(),
  dueDate: z.string().nullable().optional(),
});

const noteInputSchema = z.object({
  content: z.string().min(1),
});

export async function registerApplicationRoutes(app: FastifyInstance): Promise<void> {
  app.get('/applications', async (): Promise<Application[]> => {
    const rows = await db.select().from(applications).orderBy(desc(applications.updatedAt));
    return rows.map(rowToApplication);
  });

  app.get<{ Params: { id: string } }>('/applications/:id', async (request, reply) => {
    const appRow = await db
      .select()
      .from(applications)
      .where(eq(applications.id, request.params.id))
      .get();

    if (!appRow) return reply.code(404).send({ error: 'Application not found' });

    const jobRow = await db
      .select()
      .from(jobs)
      .where(eq(jobs.id, appRow.jobId))
      .get();

    if (!jobRow) return reply.code(500).send({ error: 'Associated job not found' });

    const resumeRow = appRow.resumeId
      ? await db.select().from(resumes).where(eq(resumes.id, appRow.resumeId)).get()
      : null;

    const noteRows = await db
      .select()
      .from(applicationNotes)
      .where(eq(applicationNotes.applicationId, appRow.id))
      .orderBy(desc(applicationNotes.createdAt));

    // Contacts linked to this application
    const appContactRows = await db
      .select()
      .from(applicationContacts)
      .where(eq(applicationContacts.applicationId, appRow.id));

    const contactIds = appContactRows.map((r) => r.contactId);
    const contactRows = contactIds.length
      ? await db.select().from(contacts).where(
          contactIds.length === 1
            ? eq(contacts.id, contactIds[0])
            : eq(contacts.id, contactIds[0]) // will filter below
        )
      : [];
    // Fetch all linked contacts (SQLite doesn't support IN easily with drizzle here)
    const contactMap = new Map<string, typeof contacts.$inferSelect>();
    for (const cid of contactIds) {
      const row = await db.select().from(contacts).where(eq(contacts.id, cid)).get();
      if (row) contactMap.set(cid, row);
    }
    void contactRows; // not used, contactMap is authoritative

    const appContacts: ApplicationContact[] = appContactRows
      .map((r) => {
        const c = contactMap.get(r.contactId);
        if (!c) return null;
        return {
          id: r.id,
          applicationId: r.applicationId,
          relationship: r.relationship as ContactRelationship,
          contact: rowToContact(c),
        };
      })
      .filter((r): r is ApplicationContact => r !== null);

    // Activities
    const activityRows = await db
      .select()
      .from(activities)
      .where(eq(activities.applicationId, appRow.id))
      .orderBy(desc(activities.happenedAt));

    // Tasks
    const taskRows = await db
      .select()
      .from(tasks)
      .where(eq(tasks.applicationId, appRow.id))
      .orderBy(desc(tasks.createdAt));

    const detail: ApplicationDetail = {
      ...rowToApplication(appRow),
      job: rowToJob(jobRow),
      resume: resumeRow ? rowToResume(resumeRow) : undefined,
      notes: noteRows.map(rowToNote),
      contacts: appContacts,
      activities: activityRows.map(rowToActivity),
      tasks: taskRows.map(rowToTask),
    };

    return detail;
  });

  app.post<{ Body: ApplicationInput }>('/applications', async (request, reply) => {
    const parsed = applicationInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid application', details: parsed.error.flatten() });
    }

    // One application per job
    const existing = await db
      .select()
      .from(applications)
      .where(eq(applications.jobId, parsed.data.jobId))
      .get();

    if (existing) {
      return reply.code(409).send({ error: 'Application for this job already exists', id: existing.id });
    }

    const now = new Date().toISOString();
    const id = crypto.randomUUID();

    // If status is 'applied' and no resumeId, auto-attach current active resume
    let resumeId: string | null = null;
    if (parsed.data.status === 'applied') {
      const activeResume = await db
        .select()
        .from(resumes)
        .where(eq(resumes.isActive, true))
        .get();
      resumeId = activeResume?.id ?? null;
    }

    await db.insert(applications).values({
      id,
      jobId: parsed.data.jobId,
      resumeId,
      status: parsed.data.status,
      appliedAt: parsed.data.appliedAt ?? null,
      updatedAt: now,
    });

    const result: Application = {
      id,
      jobId: parsed.data.jobId,
      resumeId: resumeId ?? undefined,
      status: parsed.data.status,
      appliedAt: parsed.data.appliedAt,
      updatedAt: now,
    };

    return reply.code(201).send(result);
  });

  app.patch<{ Params: { id: string }; Body: { status: Application['status'] } }>(
    '/applications/:id',
    async (request, reply) => {
      const parsed = statusPatchSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid patch', details: parsed.error.flatten() });
      }

      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();

      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const now = new Date().toISOString();
      const updates: Partial<typeof applications.$inferInsert> = { updatedAt: now };

      if (parsed.data.status !== undefined) updates.status = parsed.data.status;
      if ('dueDate' in parsed.data) updates.dueDate = parsed.data.dueDate ?? null;

      // Auto-attach active resume when transitioning to 'applied' and none set yet
      if (parsed.data.status === 'applied' && !target.resumeId) {
        const activeResume = await db
          .select()
          .from(resumes)
          .where(eq(resumes.isActive, true))
          .get();
        if (activeResume) updates.resumeId = activeResume.id;
      }

      await db.update(applications).set(updates).where(eq(applications.id, request.params.id));

      const updated = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();

      return rowToApplication(updated!);
    },
  );

  app.delete<{ Params: { id: string } }>('/applications/:id', async (request, reply) => {
    const { id } = request.params;
    const target = await db.select().from(applications).where(eq(applications.id, id)).get();
    if (!target) return reply.code(404).send({ error: 'Application not found' });

    await db.delete(applicationNotes).where(eq(applicationNotes.applicationId, id));
    await db.delete(applications).where(eq(applications.id, id));

    reply.code(204);
    return reply.send();
  });

  // Link an existing contact to an application
  app.post<{ Params: { id: string }; Body: { contactId: string; relationship: string } }>(
    '/applications/:id/contacts',
    async (request, reply) => {
      const { contactId, relationship } = request.body;
      if (!contactId || typeof contactId !== 'string') {
        return reply.code(400).send({ error: 'contactId required' });
      }

      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();
      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const contact = await db.select().from(contacts).where(eq(contacts.id, contactId)).get();
      if (!contact) return reply.code(404).send({ error: 'Contact not found' });

      // Avoid duplicate links
      const existing = await db
        .select()
        .from(applicationContacts)
        .where(eq(applicationContacts.applicationId, request.params.id))
        .all();
      if (existing.some((r) => r.contactId === contactId)) {
        return reply.code(409).send({ error: 'Contact already linked' });
      }

      const id = crypto.randomUUID();
      await db.insert(applicationContacts).values({
        id,
        applicationId: request.params.id,
        contactId,
        relationship: relationship ?? 'other',
      });

      return reply.code(201).send({
        id,
        applicationId: request.params.id,
        contact: rowToContact(contact),
        relationship: relationship ?? 'other',
      });
    },
  );

  // Unlink a contact from an application
  app.delete<{ Params: { id: string; linkId: string } }>(
    '/applications/:id/contacts/:linkId',
    async (request, reply) => {
      const link = await db
        .select()
        .from(applicationContacts)
        .where(eq(applicationContacts.id, request.params.linkId))
        .get();
      if (!link) return reply.code(404).send({ error: 'Link not found' });

      await db.delete(applicationContacts).where(eq(applicationContacts.id, request.params.linkId));
      reply.code(204);
      return reply.send();
    },
  );

  app.post<{ Params: { id: string }; Body: { content: string } }>(
    '/applications/:id/notes',
    async (request, reply) => {
      const parsed = noteInputSchema.safeParse(request.body);
      if (!parsed.success) {
        return reply.code(400).send({ error: 'Invalid note', details: parsed.error.flatten() });
      }

      const target = await db
        .select()
        .from(applications)
        .where(eq(applications.id, request.params.id))
        .get();

      if (!target) return reply.code(404).send({ error: 'Application not found' });

      const now = new Date().toISOString();
      const id = crypto.randomUUID();

      await db.insert(applicationNotes).values({
        id,
        applicationId: request.params.id,
        content: parsed.data.content,
        createdAt: now,
      });

      const note: ApplicationNote = {
        id,
        applicationId: request.params.id,
        content: parsed.data.content,
        createdAt: now,
      };

      return reply.code(201).send(note);
    },
  );
}

function rowToApplication(row: typeof applications.$inferSelect): Application {
  return {
    id: row.id,
    jobId: row.jobId,
    resumeId: row.resumeId ?? undefined,
    status: row.status as Application['status'],
    appliedAt: row.appliedAt ?? undefined,
    dueDate: row.dueDate ?? undefined,
    updatedAt: row.updatedAt,
  };
}

function rowToJob(row: typeof jobs.$inferSelect): JobPosting {
  return {
    id: row.id,
    source: row.source as JobPosting['source'],
    url: row.url ?? undefined,
    title: row.title,
    company: row.company,
    description: row.description,
    notes: row.notes,
    createdAt: row.createdAt,
  };
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

function rowToNote(row: typeof applicationNotes.$inferSelect): ApplicationNote {
  return {
    id: row.id,
    applicationId: row.applicationId,
    content: row.content,
    createdAt: row.createdAt,
  };
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
