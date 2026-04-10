import type { FastifyInstance } from 'fastify';
import { z } from 'zod';
import { eq } from 'drizzle-orm';
import type { CandidateProfile, CandidateProfileInput } from '@job-copilot/shared';
import { db } from '../db/index.js';
import { profiles, resumes } from '../db/schema.js';

// Same curated list as Market Pulse
const SKILLS = [
  'TypeScript', 'JavaScript', 'Python', 'Java', 'Go', 'Rust', 'C#', 'PHP', 'Ruby', 'Kotlin', 'Swift', 'Scala',
  'React', 'Vue', 'Angular', 'Next.js', 'Nuxt', 'Svelte', 'Redux', 'GraphQL', 'HTML', 'CSS', 'Tailwind',
  'Node.js', 'NestJS', 'Express', 'Fastify', 'Django', 'FastAPI', 'Spring', '.NET', 'Rails',
  'React Native', 'Flutter', 'iOS', 'Android',
  'PostgreSQL', 'MySQL', 'MongoDB', 'Redis', 'Elasticsearch', 'SQLite', 'Kafka', 'RabbitMQ',
  'Docker', 'Kubernetes', 'AWS', 'GCP', 'Azure', 'Terraform', 'Ansible', 'Linux', 'Nginx',
  'CI/CD', 'GitHub Actions', 'Jenkins', 'GitLab',
  'REST', 'gRPC', 'Microservices', 'Agile', 'Scrum', 'Git', 'WebSockets',
];

const profileInputSchema = z.object({
  name: z.string().min(1),
  email: z.string().email(),
  location: z.string().optional(),
  summary: z.string().optional(),
  skills: z.array(z.string()),
});

// Single-user app — store exactly one profile row with a fixed id
const PROFILE_ID = 'default';

export async function registerProfileRoutes(app: FastifyInstance): Promise<void> {
  app.get('/profile', async (_req, reply) => {
    const row = await db
      .select()
      .from(profiles)
      .where(eq(profiles.id, PROFILE_ID))
      .get();

    if (!row) return reply.code(404).send({ error: 'No profile yet' });
    return rowToProfile(row);
  });

  app.post<{ Body: CandidateProfileInput }>('/profile', async (request, reply) => {
    const parsed = profileInputSchema.safeParse(request.body);
    if (!parsed.success) {
      return reply.code(400).send({ error: 'Invalid profile', details: parsed.error.flatten() });
    }

    const now = new Date().toISOString();
    const value = {
      id: PROFILE_ID,
      name: parsed.data.name,
      email: parsed.data.email,
      location: parsed.data.location ?? null,
      summary: parsed.data.summary ?? null,
      skills: JSON.stringify(parsed.data.skills),
      updatedAt: now,
    };

    await db
      .insert(profiles)
      .values(value)
      .onConflictDoUpdate({
        target: profiles.id,
        set: {
          name: value.name,
          email: value.email,
          location: value.location,
          summary: value.summary,
          skills: value.skills,
          updatedAt: value.updatedAt,
        },
      });

    const profile: CandidateProfile = {
      id: PROFILE_ID,
      name: parsed.data.name,
      email: parsed.data.email,
      location: parsed.data.location,
      summary: parsed.data.summary,
      skills: parsed.data.skills,
      updatedAt: now,
    };

    return reply.code(200).send(profile);
  });

  // Extract skills present in the active resume
  app.get('/profile/suggested-skills', async (_req, reply) => {
    const resume = await db.select().from(resumes).where(eq(resumes.isActive, true)).get();
    if (!resume) return reply.code(200).send([]);
    const text = resume.rawText.toLowerCase();
    const found = SKILLS.filter((skill) => text.includes(skill.toLowerCase()));
    return reply.code(200).send(found);
  });
}

function rowToProfile(row: typeof profiles.$inferSelect): CandidateProfile {
  return {
    id: row.id,
    name: row.name,
    email: row.email,
    location: row.location ?? undefined,
    summary: row.summary ?? undefined,
    skills: JSON.parse(row.skills) as string[],
    updatedAt: row.updatedAt,
  };
}
