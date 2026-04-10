import type { FastifyInstance } from 'fastify';
import { sql } from 'drizzle-orm';
import type { DashboardStats, ApplicationStatus } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { and, eq } from 'drizzle-orm';
import { applications, matchResults, tasks } from '../db/schema.js';

const STATUS_KEYS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

export async function registerDashboardRoutes(app: FastifyInstance): Promise<void> {
  app.get('/dashboard/stats', async (): Promise<DashboardStats> => {
    // Total applications + group by status in one pass
    const appRows = await db
      .select({ status: applications.status })
      .from(applications);

    const total = appRows.length;
    const byStatus = Object.fromEntries(STATUS_KEYS.map((s) => [s, 0])) as Record<ApplicationStatus, number>;
    for (const row of appRows) {
      const s = row.status as ApplicationStatus;
      if (s in byStatus) byStatus[s]++;
    }

    // Average score across all match results
    const avgRow = await db
      .select({ avg: sql<number | null>`AVG(score)` })
      .from(matchResults)
      .get();

    const avgScore = avgRow?.avg != null ? Math.round(avgRow.avg) : null;

    // Top missing skills: collect all missingSkills JSON arrays, count frequency
    const skillRows = await db
      .select({ missingSkills: matchResults.missingSkills })
      .from(matchResults);

    const freq = new Map<string, number>();
    for (const row of skillRows) {
      const skills = JSON.parse(row.missingSkills) as string[];
      for (const skill of skills) {
        freq.set(skill, (freq.get(skill) ?? 0) + 1);
      }
    }

    const topMissingSkills = [...freq.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([skill, count]) => ({ skill, count }));

    // Tasks due in the next 48 h
    const cutoff = new Date(Date.now() + 48 * 60 * 60 * 1000).toISOString();
    const pendingTasks = await db
      .select({ remindAt: tasks.remindAt })
      .from(tasks)
      .where(and(eq(tasks.done, false)));
    const tasksDueSoon = pendingTasks.filter(
      (t) => t.remindAt != null && t.remindAt <= cutoff,
    ).length;

    return { total, byStatus, topMissingSkills, avgScore, tasksDueSoon };
  });
}
