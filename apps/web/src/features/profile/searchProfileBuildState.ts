import type { PersistedSearchPreferences, SearchProfileBuildResult } from '../../api/profiles';

const SEARCH_PROFILE_BUILD_STORAGE_KEY = 'job-copilot.search-profile-build.v1';

type SearchProfileBuildStore = Record<
  string,
  {
    result: SearchProfileBuildResult;
    savedAt: string;
  }
>;

function canUseStorage() {
  return typeof window !== 'undefined' && !!window.localStorage;
}

function stableHash(input: string) {
  let hash = 5381;

  for (let index = 0; index < input.length; index += 1) {
    hash = (hash * 33) ^ input.charCodeAt(index);
  }

  return (hash >>> 0).toString(36);
}

function readBuildStore(): SearchProfileBuildStore {
  if (!canUseStorage()) {
    return {};
  }

  const raw = window.localStorage.getItem(SEARCH_PROFILE_BUILD_STORAGE_KEY);
  if (!raw) {
    return {};
  }

  try {
    const parsed = JSON.parse(raw) as SearchProfileBuildStore;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

function writeBuildStore(store: SearchProfileBuildStore) {
  if (!canUseStorage()) {
    return;
  }

  window.localStorage.setItem(SEARCH_PROFILE_BUILD_STORAGE_KEY, JSON.stringify(store));
}

export function createSearchProfileBuildKey(
  profileId: string | null | undefined,
  rawText: string,
  preferences: PersistedSearchPreferences,
) {
  return stableHash(
    JSON.stringify({
      profileId: profileId ?? null,
      rawText: rawText.trim(),
      preferences,
    }),
  );
}

export function readStoredSearchProfileBuild({
  profileId,
  rawText,
  preferences,
}: {
  profileId?: string | null;
  rawText: string;
  preferences: PersistedSearchPreferences;
}) {
  const store = readBuildStore();
  const exactKey = createSearchProfileBuildKey(profileId, rawText, preferences);
  const exactMatch = store[exactKey];

  if (exactMatch) {
    return exactMatch.result;
  }

  if (!profileId) {
    return null;
  }

  const anonymousKey = createSearchProfileBuildKey(null, rawText, preferences);
  return store[anonymousKey]?.result ?? null;
}

export function writeStoredSearchProfileBuild({
  profileId,
  rawText,
  preferences,
  result,
}: {
  profileId?: string | null;
  rawText: string;
  preferences: PersistedSearchPreferences;
  result: SearchProfileBuildResult;
}) {
  const store = readBuildStore();
  const key = createSearchProfileBuildKey(profileId, rawText, preferences);

  store[key] = {
    result,
    savedAt: new Date().toISOString(),
  };

  writeBuildStore(store);
}
