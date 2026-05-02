import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import {
  cleanupExtractedResumeText,
  extractPdfTextFromData,
  extractPdfTextFromItems,
} from '../src/features/profile/profile.pdf.utils';

const FIXTURES_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), './fixtures');

describe('extractPdfTextFromItems', () => {
  it('keeps tight fragments together and preserves line breaks', () => {
    const text = extractPdfTextFromItems([
      {
        str: 'The energy conversion ef',
        width: 118,
        height: 12,
        transform: [1, 0, 0, 1, 72, 720],
      },
      {
        str: 'fi',
        width: 8,
        height: 12,
        transform: [1, 0, 0, 1, 190, 720],
      },
      {
        str: 'ciency',
        width: 34,
        height: 12,
        hasEOL: true,
        transform: [1, 0, 0, 1, 198, 720],
      },
      {
        str: 'React',
        width: 36,
        height: 12,
        transform: [1, 0, 0, 1, 72, 696],
      },
      {
        str: 'Native',
        width: 40,
        height: 12,
        hasEOL: true,
        transform: [1, 0, 0, 1, 116, 696],
      },
    ]);

    expect(text).toContain('efficiency');
    expect(text).toContain('React Native');
    expect(text).toContain('\n');
    expect(text).not.toContain('ef fi ciency');
  });
});

describe('cleanupExtractedResumeText', () => {
  it('repairs hyphenated line wraps and letter-spaced seniority markers', () => {
    const cleaned = cleanupExtractedResumeText(
      [
        'S e n i o r Front-',
        'end React Na-',
        'tive Engineer',
        '',
        'Full-',
        'stack Developer',
        'L e a d mentor',
      ].join('\n'),
    );

    expect(cleaned).toContain('Senior');
    expect(cleaned).toContain('Frontend React Native Engineer');
    expect(cleaned).toContain('Fullstack Developer');
    expect(cleaned).toContain('Lead mentor');
  });
});

describe('PDF fixtures', () => {
  it('keeps resume phrases recoverable from a fragmented resume PDF', async () => {
    const text = await extractFixtureText('resume-hyphenated.pdf');

    expect(text).toContain('Summary');
    expect(text).toContain('Senior');
    expect(text).toContain('Frontend');
    expect(text).toContain('React Native');
    expect(text).toContain('Fullstack');
    expect(text).toContain('Lead');
    expect(text).toContain('\n');
  });

  it('cleans spacing artifacts from a representative noisy PDF', async () => {
    const text = await extractFixtureText('spacing-artifacts.pdf');

    expect(text).toContain('Senior');
    expect(text).toContain('Lead');
    expect(text).toContain('efficiency');
    expect(text).not.toContain('S e n i o r');
    expect(text).not.toContain('ef fi ciency');
  });
});

async function extractFixtureText(filename: string): Promise<string> {
  const data = new Uint8Array(await readFile(path.join(FIXTURES_DIR, filename)));
  return cleanupExtractedResumeText(await extractPdfTextFromData(data));
}
