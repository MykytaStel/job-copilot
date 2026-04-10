import { parse, type HTMLElement } from 'node-html-parser';

export interface ScrapedJob {
  title: string;
  company: string;
  description: string;
}

export async function fetchJobFromUrl(url: string): Promise<ScrapedJob> {
  const response = await fetch(url, {
    headers: {
      'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
      Accept: 'text/html,application/xhtml+xml',
      'Accept-Language': 'uk,en-US;q=0.9,en;q=0.8',
    },
    signal: AbortSignal.timeout(12_000),
  });

  if (!response.ok) throw new Error(`HTTP ${response.status}`);

  const html = await response.text();
  return parseJob(html, url);
}

function parseJob(html: string, url: string): ScrapedJob {
  const root = parse(html);
  for (const el of root.querySelectorAll('script, style, noscript, nav, footer, aside')) {
    el.remove();
  }

  if (url.includes('djinni.co')) return parseDjinni(root);
  if (url.includes('work.ua')) return parseWorkUa(root);
  if (url.includes('robota.ua')) return parseRobotaUa(root);
  return parseGeneric(root);
}

function parseDjinni(root: HTMLElement): ScrapedJob {
  const title =
    root.querySelector('h1.job-details--title')?.text.trim() ||
    root.querySelector('h1')?.text.trim() ||
    '';

  const company =
    root.querySelector('.job-details--title a')?.text.trim() ||
    root.querySelector('a[href*="/company"]')?.text.trim() ||
    '';

  const description =
    root.querySelector('.job-details--description-text')?.text.trim() ||
    root.querySelector('[class*="description"]')?.text.trim() ||
    '';

  return { title, company, description };
}

function parseWorkUa(root: HTMLElement): ScrapedJob {
  const title = root.querySelector('h1')?.text.trim() || '';

  const company =
    root.querySelector('a[href*="/employer"]')?.text.trim() ||
    root.querySelector('.employer-card a')?.text.trim() ||
    '';

  const description =
    root.querySelector('#job-description')?.text.trim() ||
    root.querySelector('.b-typo.vacancy-text')?.text.trim() ||
    root.querySelector('[id*="description"]')?.text.trim() ||
    '';

  return { title, company, description };
}

function parseRobotaUa(root: HTMLElement): ScrapedJob {
  // robota.ua is an Angular SPA — try meta tags first since HTML may be incomplete
  const metaTitle =
    root.querySelector('meta[property="og:title"]')?.getAttribute('content') ||
    root.querySelector('title')?.text.trim() ||
    '';

  const title = root.querySelector('h1')?.text.trim() || metaTitle;

  const company =
    root.querySelector('[class*="company-name"]')?.text.trim() ||
    root.querySelector('a[href*="/employer"]')?.text.trim() ||
    '';

  const description =
    root.querySelector('[class*="vacancy-description"]')?.text.trim() ||
    root.querySelector('[class*="description"]')?.text.trim() ||
    '';

  return { title, company, description };
}

function parseGeneric(root: HTMLElement): ScrapedJob {
  const title =
    root.querySelector('h1')?.text.trim() ||
    root.querySelector('title')?.text.trim() ||
    '';

  let description = '';
  for (const el of root.querySelectorAll(
    'article, main, [class*="description"], [class*="vacancy"], [class*="job-detail"]',
  )) {
    const text = el.text.trim();
    if (text.length > description.length) description = text;
  }

  // Fallback: grab body text, capped at 5000 chars
  if (description.length < 100) {
    description = (root.querySelector('main')?.text || root.querySelector('body')?.text || '').trim().slice(0, 5000);
  }

  return { title, company: '', description };
}
