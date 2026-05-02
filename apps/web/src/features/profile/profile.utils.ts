import type {
  RoleCatalogItem,
  SearchProfileBuildResult,
  SourceCatalogItem,
} from '../../api/profiles';
import { formatFallbackLabel } from '../../lib/format';

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
    sources.find((source) => source.id === sourceId)?.displayName ?? formatFallbackLabel(sourceId)
  );
}

export function getFitScoreTone(score: number) {
  if (score >= 80) return 'high';
  if (score >= 60) return 'medium';
  return 'low';
}

export type BuiltSearchProfile = SearchProfileBuildResult['searchProfile'];
