import type { SearchProfileBuildResult } from '../../api/profiles';

const LEGACY_SEARCH_PROFILE_DRAFT_STORAGE_KEY = 'job-copilot.search-profile-draft.v1';
const SEARCH_PROFILE_DRAFT_STORAGE_KEY = 'job-copilot.search-profile-draft.v2';

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

type SearchProfileDraftStore = {
  anonymous?: SearchProfileDraft;
  byProfileId?: Record<string, SearchProfileDraft>;
};

function normalizeDraft(parsed: Partial<SearchProfileDraft>): SearchProfileDraft {
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
}

function readStoredValue<T>(key: string): T | null {
  if (!canUseStorage()) {
    return null;
  }

  const raw = window.localStorage.getItem(key);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

function readDraftStore(): SearchProfileDraftStore {
  const store = readStoredValue<SearchProfileDraftStore>(SEARCH_PROFILE_DRAFT_STORAGE_KEY);
  return store && typeof store === 'object' ? store : {};
}

export function readSearchProfileDraft(profileId?: string | null): SearchProfileDraft | null {
  const store = readDraftStore();

  if (profileId && store.byProfileId?.[profileId]) {
    return normalizeDraft(store.byProfileId[profileId]);
  }

  if (!profileId && store.anonymous) {
    return normalizeDraft(store.anonymous);
  }

  const legacyDraft = readStoredValue<Partial<SearchProfileDraft>>(
    LEGACY_SEARCH_PROFILE_DRAFT_STORAGE_KEY,
  );

  return legacyDraft ? normalizeDraft(legacyDraft) : null;
}

export function writeSearchProfileDraft(draft: SearchProfileDraft, profileId?: string | null) {
  if (!canUseStorage()) {
    return;
  }

  const store = readDraftStore();

  if (profileId) {
    store.byProfileId = {
      ...(store.byProfileId ?? {}),
      [profileId]: draft,
    };
  } else {
    store.anonymous = draft;
  }

  window.localStorage.setItem(SEARCH_PROFILE_DRAFT_STORAGE_KEY, JSON.stringify(store));
  window.localStorage.removeItem(LEGACY_SEARCH_PROFILE_DRAFT_STORAGE_KEY);
}

export function clearSearchProfileDraft(profileId?: string | null) {
  if (!canUseStorage()) {
    return;
  }

  const store = readDraftStore();

  if (profileId) {
    if (store.byProfileId?.[profileId]) {
      delete store.byProfileId[profileId];
    }
  } else {
    delete store.anonymous;
  }

  window.localStorage.setItem(SEARCH_PROFILE_DRAFT_STORAGE_KEY, JSON.stringify(store));
  window.localStorage.removeItem(LEGACY_SEARCH_PROFILE_DRAFT_STORAGE_KEY);
}
