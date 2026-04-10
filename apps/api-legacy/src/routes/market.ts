import type { FastifyInstance } from 'fastify';
import { eq } from 'drizzle-orm';
import { db } from '../db/index.js';
import { jobs, resumes } from '../db/schema.js';

// Curated skill list for the Ukrainian IT market
const SKILLS = [
  // Languages
  'TypeScript', 'JavaScript', 'Python', 'Java', 'Go', 'Rust', 'C#', 'PHP', 'Ruby', 'Kotlin', 'Swift', 'Scala',
  // Frontend
  'React', 'Vue', 'Angular', 'Next.js', 'Nuxt', 'Svelte', 'Redux', 'GraphQL', 'HTML', 'CSS', 'Tailwind',
  // Backend
  'Node.js', 'NestJS', 'Express', 'Fastify', 'Django', 'FastAPI', 'Spring', '.NET', 'Rails',
  // Mobile
  'React Native', 'Flutter', 'iOS', 'Android',
  // Data & Storage
  'PostgreSQL', 'MySQL', 'MongoDB', 'Redis', 'Elasticsearch', 'SQLite', 'Kafka', 'RabbitMQ',
  // DevOps & Cloud
  'Docker', 'Kubernetes', 'AWS', 'GCP', 'Azure', 'Terraform', 'Ansible', 'Linux', 'Nginx',
  'CI/CD', 'GitHub Actions', 'Jenkins', 'GitLab',
  // Architecture & Practices
  'REST', 'gRPC', 'Microservices', 'Agile', 'Scrum', 'Git', 'WebSockets',
];

// Salary patterns common in Ukrainian job postings
const SALARY_RE = [
  /\$\s?\d[\d\s]*(?:\s*[-–—]\s*\$?\s?\d[\d\s]*)?/g,          // $3000 - $5000
  /\d[\d\s]*\s*(?:USD|usd)\b/g,                                // 3000 USD
  /від\s+\d[\d.,\s]+(?:до\s+\d[\d.,\s]+)?\s*(?:грн|₴|USD|\$|тис\.?)/gi,
  /\d+\s*[kк]\s*[-–—]\s*\d+\s*[kк]/gi,                       // 3k-5k
];

export interface SkillStat {
  skill: string;
  count: number;       // how many jobs mention it
  pct: number;         // % of all saved jobs
  inResume: boolean;
}

export interface MarketInsights {
  totalJobs: number;
  coverageScore: number;           // % of top-10 skills the candidate has
  topSkills: SkillStat[];          // top 20 by demand
  hotGaps: string[];               // skills missing from resume, appearing in 30%+ of jobs
  salaryMentions: string[];
}

export async function registerMarketRoutes(app: FastifyInstance): Promise<void> {
  app.get('/market/insights', async (_req, reply) => {
    const allJobs = await db.select().from(jobs);
    if (allJobs.length === 0) {
      return reply.code(200).send({
        totalJobs: 0,
        coverageScore: 0,
        topSkills: [],
        hotGaps: [],
        salaryMentions: [],
      } satisfies MarketInsights);
    }

    const resume = await db.select().from(resumes).where(eq(resumes.isActive, true)).get();
    const resumeText = resume?.rawText.toLowerCase() ?? '';

    const total = allJobs.length;
    const allDescriptions = allJobs.map((j) => j.description.toLowerCase());
    const combined = allDescriptions.join('\n');

    // Score each skill
    const stats: SkillStat[] = SKILLS.map((skill) => {
      const lower = skill.toLowerCase();
      const count = allDescriptions.filter((d) => d.includes(lower)).length;
      return {
        skill,
        count,
        pct: Math.round((count / total) * 100),
        inResume: resumeText.includes(lower),
      };
    })
      .filter((s) => s.count > 0)
      .sort((a, b) => b.count - a.count)
      .slice(0, 20);

    const top10 = stats.slice(0, 10);
    const coverageScore =
      top10.length === 0
        ? 0
        : Math.round((top10.filter((s) => s.inResume).length / top10.length) * 100);

    const hotGaps = stats
      .filter((s) => !s.inResume && s.pct >= 30)
      .map((s) => s.skill);

    // Extract salary mentions
    const salarySet = new Set<string>();
    for (const re of SALARY_RE) {
      const matches = combined.match(re) ?? [];
      matches.forEach((m) => salarySet.add(m.trim()));
    }

    return {
      totalJobs: total,
      coverageScore,
      topSkills: stats,
      hotGaps,
      salaryMentions: [...salarySet].slice(0, 15),
    } satisfies MarketInsights;
  });
}
