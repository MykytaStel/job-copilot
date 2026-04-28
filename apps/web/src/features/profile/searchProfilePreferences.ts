import {
  DEFAULT_SCORING_WEIGHTS,
  type PersistedSearchPreferences,
} from '../../api/profiles';
import { parseKeywordInput } from './profile.utils';
import type { SearchProfileDraft } from './searchProfileDraft';

export function buildSearchProfileDraftFromPreferences(
  preferences?: PersistedSearchPreferences | null,
): SearchProfileDraft | null {
  if (!preferences) {
    return null;
  }

  return {
    targetRegions: preferences.targetRegions,
    workModes: preferences.workModes,
    preferredRoles: preferences.preferredRoles,
    allowedSources: preferences.allowedSources,
    includeKeywordsInput: preferences.includeKeywords.join(', '),
    excludeKeywordsInput: preferences.excludeKeywords.join(', '),
  };
}

export function buildPersistedSearchPreferences(
  draft: SearchProfileDraft,
  existingPreferences?: PersistedSearchPreferences | null,
): PersistedSearchPreferences {
  const preferences: PersistedSearchPreferences = {
    targetRegions: draft.targetRegions,
    workModes: draft.workModes,
    preferredRoles: draft.preferredRoles,
    allowedSources: draft.allowedSources,
    includeKeywords: parseKeywordInput(draft.includeKeywordsInput),
    excludeKeywords: parseKeywordInput(draft.excludeKeywordsInput),
  };

  if (existingPreferences?.scoringWeights) {
    preferences.scoringWeights = existingPreferences.scoringWeights;
  }

  return preferences;
}

export function resolveSearchProfileDraft(
  preferences?: PersistedSearchPreferences | null,
  localDraft?: SearchProfileDraft | null,
): SearchProfileDraft | null {
  return buildSearchProfileDraftFromPreferences(preferences) ?? localDraft ?? null;
}
