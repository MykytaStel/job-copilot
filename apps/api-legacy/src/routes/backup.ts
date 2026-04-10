import type { FastifyInstance } from 'fastify';
import { db } from '../db/index.js';
import {
  activities,
  alerts,
  applicationContacts,
  applicationNotes,
  applications,
  contacts,
  coverLetters,
  interviewQA,
  jobs,
  matchResults,
  offers,
  profiles,
  resumes,
  tasks,
} from '../db/schema.js';

export async function registerBackupRoutes(app: FastifyInstance): Promise<void> {
  app.get('/backup', async () => {
    const [
      allProfiles,
      allJobs,
      allResumes,
      allMatchResults,
      allApplications,
      allNotes,
      allContacts,
      allAppContacts,
      allActivities,
      allTasks,
      allAlerts,
      allLetters,
      allQA,
      allOffers,
    ] = await Promise.all([
      db.select().from(profiles),
      db.select().from(jobs),
      db.select().from(resumes),
      db.select().from(matchResults),
      db.select().from(applications),
      db.select().from(applicationNotes),
      db.select().from(contacts),
      db.select().from(applicationContacts),
      db.select().from(activities),
      db.select().from(tasks),
      db.select().from(alerts),
      db.select().from(coverLetters),
      db.select().from(interviewQA),
      db.select().from(offers),
    ]);

    return {
      version: '1',
      exportedAt: new Date().toISOString(),
      profiles: allProfiles,
      jobs: allJobs,
      resumes: allResumes,
      matchResults: allMatchResults,
      applications: allApplications,
      applicationNotes: allNotes,
      contacts: allContacts,
      applicationContacts: allAppContacts,
      activities: allActivities,
      tasks: allTasks,
      alerts: allAlerts,
      coverLetters: allLetters,
      interviewQA: allQA,
      offers: allOffers,
    };
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  app.post<{ Body: Record<string, any> }>('/restore', async (request, reply) => {
    const data = request.body;
    if (!data?.version || !data?.exportedAt) {
      return reply.code(400).send({ error: 'Invalid backup: missing version or exportedAt' });
    }

    // Delete in reverse dependency order (no FK enforcement in SQLite by default, but be safe)
    await db.delete(offers);
    await db.delete(interviewQA);
    await db.delete(coverLetters);
    await db.delete(tasks);
    await db.delete(activities);
    await db.delete(applicationContacts);
    await db.delete(applicationNotes);
    await db.delete(applications);
    await db.delete(matchResults);
    await db.delete(alerts);
    await db.delete(contacts);
    await db.delete(resumes);
    await db.delete(jobs);
    await db.delete(profiles);

    // Re-insert each table if data exists
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    async function insertIf(table: any, rows: any[]) {
      if (rows?.length) await db.insert(table).values(rows);
    }

    await insertIf(profiles, data.profiles);
    await insertIf(jobs, data.jobs);
    await insertIf(resumes, data.resumes);
    await insertIf(matchResults, data.matchResults);
    await insertIf(applications, data.applications);
    await insertIf(applicationNotes, data.applicationNotes);
    await insertIf(contacts, data.contacts);
    await insertIf(applicationContacts, data.applicationContacts);
    await insertIf(activities, data.activities);
    await insertIf(tasks, data.tasks);
    await insertIf(alerts, data.alerts);
    await insertIf(coverLetters, data.coverLetters);
    await insertIf(interviewQA, data.interviewQA);
    await insertIf(offers, data.offers);

    return { restored: true, exportedAt: data.exportedAt as string };
  });
}
