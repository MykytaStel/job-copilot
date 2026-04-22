import type { SearchProfileBuildResult } from '../../api/profiles';

const SEARCH_PROFILE_DRAFT_STORAGE_KEY = 'job-copilot.search-profile-draft.v1';

export type SearchProfileDraft = {
  targetRegions: SearchProfileBuildResult['searchProfile']['targetRegions'];
  workModes: SearchProfileBuildResult['searchProfile']['workModes'];
  preferredRoles: string[];
  allowedSources: string[];
  includeKeywordsInput: string;
  excludeKeywordsInput: string;
};

function canUseStorage() {
  return typeof window !== 'undefined' && !!window.localStorage;
}

export function readSearchProfileDraft(): SearchProfileDraft | null {
  if (!canUseStorage()) {
    return null;
  }

  const raw = window.localStorage.getItem(SEARCH_PROFILE_DRAFT_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<SearchProfileDraft>;
    return {
      targetRegions: Array.isArray(parsed.targetRegions) ? parsed.targetRegions : [],
      workModes: Array.isArray(parsed.workModes) ? parsed.workModes : [],
      preferredRoles: Array.isArray(parsed.preferredRoles) ? parsed.preferredRoles : [],
      allowedSources: Array.isArray(parsed.allowedSources) ? parsed.allowedSources : [],
      includeKeywordsInput:
        typeof parsed.includeKeywordsInput === 'string' ? parsed.includeKeywordsInput : '',
      excludeKeywordsInput:
        typeof parsed.excludeKeywordsInput === 'string' ? parsed.excludeKeywordsInput : '',
    };
  } catch {
    return null;
  }
}

export function writeSearchProfileDraft(draft: SearchProfileDraft) {
  if (!canUseStorage()) {
    return;
  }

  window.localStorage.setItem(SEARCH_PROFILE_DRAFT_STORAGE_KEY, JSON.stringify(draft));
}

export function clearSearchProfileDraft() {
  if (!canUseStorage()) {
    return;
  }

  window.localStorage.removeItem(SEARCH_PROFILE_DRAFT_STORAGE_KEY);
}
