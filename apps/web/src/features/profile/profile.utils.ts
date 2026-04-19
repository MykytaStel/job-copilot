import PdfjsWorker from 'pdfjs-dist/legacy/build/pdf.worker.mjs?worker&inline';

import type {
  RoleCatalogItem,
  SearchProfileBuildResult,
  SourceCatalogItem,
} from '../../api';
import { formatFallbackLabel } from '../../lib/format';

type PdfTextItemLike = {
  str: string;
  width: number;
  height: number;
  hasEOL?: boolean;
  transform: number[];
};

type PdfLine = {
  fragments: PdfTextItemLike[];
  y: number;
  height: number;
};

const LETTER_SPACED_WORDS = new Set([
  'backend',
  'developer',
  'engineer',
  'frontend',
  'fullstack',
  'junior',
  'lead',
  'manager',
  'middle',
  'native',
  'principal',
  'react',
  'senior',
  'staff',
  'typescript',
]);

const pdfjsLibPromise = import('pdfjs-dist/legacy/build/pdf.mjs');

export async function extractPdfText(file: File): Promise<string> {
  const pdfjsLib = await pdfjsLibPromise;
  if (!pdfjsLib.GlobalWorkerOptions.workerPort) {
    pdfjsLib.GlobalWorkerOptions.workerPort = new PdfjsWorker();
  }

  const text = await extractPdfTextFromData(new Uint8Array(await file.arrayBuffer()));

  return cleanupExtractedResumeText(text);
}

export async function extractPdfTextFromData(data: Uint8Array): Promise<string> {
  const pdfjsLib = await pdfjsLibPromise;
  const pdf = await pdfjsLib
    .getDocument({
      data,
      ...(typeof window === 'undefined'
        ? {
            disableFontFace: true,
            useSystemFonts: true,
          }
        : {}),
    })
    .promise;
  const pages: string[] = [];

  for (let index = 1; index <= pdf.numPages; index += 1) {
    const page = await pdf.getPage(index);
    const content = await page.getTextContent();
    const pageText = extractPdfTextFromItems(
      content.items.flatMap((item) =>
        'str' in item
          ? [{
              str: item.str,
              width: item.width,
              height: item.height,
              hasEOL: item.hasEOL,
              transform: item.transform,
            }]
          : [],
      ),
    );
    pages.push(pageText);
  }

  return pages.join('\n\n');
}

export function extractPdfTextFromItems(items: PdfTextItemLike[]): string {
  const lines: PdfLine[] = [];
  let currentLine: PdfLine | null = null;

  for (const item of items) {
    if (!item.str) {
      if (item.hasEOL && currentLine) {
        lines.push(currentLine);
        currentLine = null;
      }
      continue;
    }

    if (!currentLine) {
      currentLine = createLine(item);
    } else if (shouldAppendToLine(currentLine, item)) {
      currentLine.fragments.push(item);
      currentLine.y = (currentLine.y + item.transform[5]) / 2;
      currentLine.height = Math.max(currentLine.height, item.height || 0);
    } else {
      lines.push(currentLine);
      currentLine = createLine(item);
    }

    if (item.hasEOL && currentLine) {
      lines.push(currentLine);
      currentLine = null;
    }
  }

  if (currentLine) {
    lines.push(currentLine);
  }

  return joinPdfLines(lines);
}

export function cleanupExtractedResumeText(value: string): string {
  let cleaned = value
    .replace(/\r\n?/g, '\n')
    .replace(/\u00a0/g, ' ')
    .replace(/[^\S\n]+/g, ' ')
    .replace(/([\p{L}\p{N}])-\s*\n\s*([\p{L}\p{N}])/gu, '$1$2')
    .replace(/[ \t]+\n/g, '\n')
    .replace(/\n[ \t]+/g, '\n')
    .replace(/([\p{Ll}\p{N},.)])\n(?=[\p{Ll}\p{N}])/gu, '$1 ');

  cleaned = cleaned
    .split('\n')
    .map((line) => cleanupExtractedResumeLine(line))
    .join('\n');

  return cleaned.replace(/\n{3,}/g, '\n\n').trim();
}

export function toggleValue<T>(current: T[], value: T): T[] {
  return current.includes(value)
    ? current.filter((existing) => existing !== value)
    : [...current, value];
}

export function parseKeywordInput(value: string): string[] {
  const keywords: string[] = [];

  for (const item of value.split(/[\n,]/)) {
    const normalized = item.trim();

    if (normalized && !keywords.includes(normalized)) {
      keywords.push(normalized);
    }
  }

  return keywords;
}

export function resolveRoleLabel(roles: RoleCatalogItem[], roleId: string): string {
  return roles.find((role) => role.id === roleId)?.displayName ?? formatFallbackLabel(roleId);
}

export function resolveSourceLabel(sources: SourceCatalogItem[], sourceId: string): string {
  return (
    sources.find((source) => source.id === sourceId)?.displayName ??
    formatFallbackLabel(sourceId)
  );
}

export function getFitScoreTone(score: number) {
  if (score >= 80) return 'high';
  if (score >= 60) return 'medium';
  return 'low';
}

export type BuiltSearchProfile = SearchProfileBuildResult['searchProfile'];

function createLine(item: PdfTextItemLike): PdfLine {
  return {
    fragments: [item],
    y: item.transform[5],
    height: item.height || 0,
  };
}

function shouldAppendToLine(line: PdfLine, item: PdfTextItemLike): boolean {
  const lastFragment = line.fragments[line.fragments.length - 1];
  if (lastFragment.hasEOL) {
    return false;
  }

  const yDelta = Math.abs(lastFragment.transform[5] - item.transform[5]);
  const heightTolerance = Math.max(line.height, item.height || 0, 8) * 0.75;

  return yDelta <= heightTolerance;
}

function joinPdfLines(lines: PdfLine[]): string {
  const renderedLines = lines
    .map((line) => ({
      text: joinPdfLineFragments(line.fragments).trim(),
      y: line.y,
      height: line.height || 0,
    }))
    .filter((line) => line.text);

  if (renderedLines.length === 0) {
    return '';
  }

  let text = renderedLines[0].text;

  for (let index = 1; index < renderedLines.length; index += 1) {
    const previous = renderedLines[index - 1];
    const current = renderedLines[index];
    const verticalGap = Math.abs(previous.y - current.y);
    const paragraphBreak =
      verticalGap > Math.max(previous.height, current.height, 12) * 1.45;

    text += paragraphBreak ? '\n\n' : '\n';
    text += current.text;
  }

  return text;
}

function joinPdfLineFragments(fragments: PdfTextItemLike[]): string {
  if (fragments.length === 0) {
    return '';
  }

  let text = fragments[0].str;

  for (let index = 1; index < fragments.length; index += 1) {
    const previous = fragments[index - 1];
    const current = fragments[index];
    const previousEndX = previous.transform[4] + previous.width;
    const gap = current.transform[4] - previousEndX;

    if (shouldInsertSpace(previous.str, current.str, gap, current.height || 0)) {
      text += ' ';
    }

    text += current.str;
  }

  return text;
}

function shouldInsertSpace(
  previous: string,
  current: string,
  gap: number,
  height: number,
): boolean {
  if (!previous || !current) {
    return false;
  }

  if (/\s$/.test(previous) || /^\s/.test(current)) {
    return false;
  }

  if (/[([{/"'“‘-]$/.test(previous)) {
    return false;
  }

  if (/^[,.;:!?%)\]}”’"']/.test(current)) {
    return false;
  }

  return gap > Math.max(height * 0.18, 1.5);
}

function cleanupExtractedResumeLine(line: string): string {
  if (!line.trim()) {
    return '';
  }

  return collapseKnownLetterSpacing(
    collapseKnownLigatureSplits(line).replace(/ {2,}/g, ' ').trim(),
  );
}

function collapseKnownLigatureSplits(line: string): string {
  return line.replace(
    /\b([\p{L}]{2,}) (ffi|ffl|ff|fi|fl) ([\p{L}]{2,})\b/giu,
    '$1$2$3',
  );
}

function collapseKnownLetterSpacing(line: string): string {
  return line.replace(/\b(?:[\p{L}]\s){3,}[\p{L}]\b/gu, (match) => {
    const collapsed = match.replace(/\s+/g, '');

    return LETTER_SPACED_WORDS.has(collapsed.toLowerCase()) ? collapsed : match;
  });
}
