import PdfjsWorker from 'pdfjs-dist/build/pdf.worker.mjs?worker&inline';

import type {
  RoleCatalogItem,
  SearchProfileBuildResult,
  SourceCatalogItem,
} from '../../api';
import { formatFallbackLabel } from '../../lib/format';

export async function extractPdfText(file: File): Promise<string> {
  const pdfjsLib = await import('pdfjs-dist');
  if (!pdfjsLib.GlobalWorkerOptions.workerPort) {
    pdfjsLib.GlobalWorkerOptions.workerPort = new PdfjsWorker();
  }

  const buffer = await file.arrayBuffer();
  const pdf = await pdfjsLib.getDocument({ data: buffer }).promise;
  const pages: string[] = [];

  for (let index = 1; index <= pdf.numPages; index += 1) {
    const page = await pdf.getPage(index);
    const content = await page.getTextContent();
    const pageText = content.items
      .map((item) => ('str' in item ? item.str : ''))
      .join(' ');
    pages.push(pageText);
  }

  return pages.join('\n\n');
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
