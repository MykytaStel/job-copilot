import { Telegraf } from 'telegraf';
import { desc, eq, gte, and } from 'drizzle-orm';
import { db } from '../db/index.js';
import { applications, jobs, tasks } from '../db/schema.js';

const BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN;

const HELP_TEXT = [
  'Job Copilot UA — commands:',
  '',
  '/status   — application pipeline stats',
  '/jobs     — 5 most recent jobs',
  '/tasks    — upcoming tasks',
  '/help     — this message',
].join('\n');

function createBot(): Telegraf | null {
  if (!BOT_TOKEN) return null;

  const bot = new Telegraf(BOT_TOKEN);

  bot.start((ctx) => ctx.reply(HELP_TEXT));
  bot.help((ctx) => ctx.reply(HELP_TEXT));

  bot.command('status', async (ctx) => {
    const all = await db.select().from(applications);
    const counts: Record<string, number> = {};
    for (const a of all) {
      counts[a.status] = (counts[a.status] ?? 0) + 1;
    }
    const STATUS_ORDER = ['saved', 'applied', 'interview', 'offer', 'rejected'];
    const lines = ['📊 Pipeline:', ''];
    for (const s of STATUS_ORDER) {
      if (counts[s]) lines.push(`${s}: ${counts[s]}`);
    }
    lines.push('', `Total: ${all.length}`);
    return ctx.reply(lines.join('\n'));
  });

  bot.command('jobs', async (ctx) => {
    const recent = await db.select().from(jobs).orderBy(desc(jobs.createdAt)).limit(5);
    if (!recent.length) return ctx.reply('No jobs saved yet.');
    const lines = ['💼 Recent jobs:', ''];
    for (const j of recent) {
      lines.push(`• ${j.title} @ ${j.company}`);
      if (j.url) lines.push(`  ${j.url}`);
    }
    return ctx.reply(lines.join('\n'));
  });

  bot.command('tasks', async (ctx) => {
    const now = new Date().toISOString();
    const upcoming = await db
      .select()
      .from(tasks)
      .where(and(eq(tasks.done, false), gte(tasks.remindAt, now)))
      .orderBy(tasks.remindAt)
      .limit(10);

    if (!upcoming.length) return ctx.reply('No upcoming tasks.');
    const lines = ['✅ Upcoming tasks:', ''];
    for (const t of upcoming) {
      const date = t.remindAt ? ` (${t.remindAt.slice(0, 10)})` : '';
      lines.push(`• ${t.title}${date}`);
    }
    return ctx.reply(lines.join('\n'));
  });

  return bot;
}

export const bot = createBot();

/** Called by the webhook route — hands the raw Telegram update to the bot. */
export async function handleWebhook(body: unknown): Promise<void> {
  if (!bot) return;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await bot.handleUpdate(body as any);
}

/**
 * Registers the webhook URL with Telegram.
 * Call once after deploying to a public HTTPS endpoint.
 */
export async function setupWebhook(webhookUrl: string): Promise<void> {
  if (!BOT_TOKEN) throw new Error('TELEGRAM_BOT_TOKEN is not set');
  const res = await fetch(`https://api.telegram.org/bot${BOT_TOKEN}/setWebhook`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url: webhookUrl }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(`Telegram setWebhook failed: ${JSON.stringify(body)}`);
  }
}
