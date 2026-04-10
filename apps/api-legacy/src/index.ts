import 'dotenv/config';
import Fastify from 'fastify';
import cors from '@fastify/cors';
import multipart from '@fastify/multipart';
import { registerHealthRoutes } from './routes/health.js';
import { registerJobRoutes } from './routes/jobs.js';
import { registerProfileRoutes } from './routes/profile.js';
import { registerResumeRoutes } from './routes/resume.js';
import { registerMatchRoutes } from './routes/match.js';
import { registerApplicationRoutes } from './routes/applications.js';
import { registerDashboardRoutes } from './routes/dashboard.js';
import { registerMarketRoutes } from './routes/market.js';
import { registerAlertRoutes } from './routes/alerts.js';
import { registerContactRoutes } from './routes/contacts.js';
import { registerActivityRoutes } from './routes/activities.js';
import { registerTaskRoutes } from './routes/tasks.js';
import { registerSearchRoutes } from './routes/search.js';
import { registerCoverLetterRoutes } from './routes/cover-letters.js';
import { registerInterviewQARoutes } from './routes/interview-qa.js';
import { registerOfferRoutes } from './routes/offers.js';
import { registerBackupRoutes } from './routes/backup.js';
import { registerImportRoutes } from './routes/import.js';
import { registerTelegramRoutes } from './routes/telegram.js';

const port = Number(process.env.PORT ?? 3001);
const host = process.env.HOST ?? '0.0.0.0';

async function buildServer() {
  const app = Fastify({ logger: true });

  await app.register(cors, {
    origin: true,
    methods: ['GET', 'POST', 'PATCH', 'PUT', 'DELETE', 'OPTIONS'],
  });
  await app.register(multipart, { limits: { fileSize: 10 * 1024 * 1024 } }); // 10 MB

  await registerHealthRoutes(app);
  await registerJobRoutes(app);
  await registerProfileRoutes(app);
  await registerResumeRoutes(app);
  await registerMatchRoutes(app);
  await registerApplicationRoutes(app);
  await registerDashboardRoutes(app);
  await registerMarketRoutes(app);
  await registerAlertRoutes(app);
  await registerContactRoutes(app);
  await registerActivityRoutes(app);
  await registerTaskRoutes(app);
  await registerSearchRoutes(app);
  await registerCoverLetterRoutes(app);
  await registerInterviewQARoutes(app);
  await registerOfferRoutes(app);
  await registerBackupRoutes(app);
  await registerImportRoutes(app);
  await registerTelegramRoutes(app);

  return app;
}

async function start(): Promise<void> {
  const app = await buildServer();

  try {
    await app.listen({ port, host });
    app.log.info(`API running on http://${host}:${port}`);
  } catch (error) {
    app.log.error(error);
    process.exit(1);
  }
}

void start();
