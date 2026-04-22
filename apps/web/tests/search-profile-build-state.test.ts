import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  readStoredSearchProfileBuild,
  writeStoredSearchProfileBuild,
} from '../src/features/profile/searchProfileBuildState';

function createStorage() {
  const data = new Map<string, string>();

  return {
    getItem(key: string) {
      return data.has(key) ? data.get(key)! : null;
    },
    setItem(key: string, value: string) {
      data.set(key, value);
    },
    removeItem(key: string) {
      data.delete(key);
    },
  };
}

const preferences = {
  targetRegions: ['ua'] as const,
  workModes: ['remote'] as const,
  preferredRoles: ['frontend_engineer'],
  allowedSources: ['djinni'],
  includeKeywords: ['react'],
  excludeKeywords: ['wordpress'],
};

const buildResult = {
  analyzedProfile: {
    summary: 'Frontend profile',
    primaryRole: 'frontend_engineer',
    seniority: 'senior',
    skills: ['react', 'typescript'],
    keywords: ['frontend'],
    roleCandidates: [],
    suggestedSearchTerms: ['react'],
  },
  searchProfile: {
    primaryRole: 'frontend_engineer',
    primaryRoleConfidence: 91,
    targetRoles: ['frontend_engineer'],
    roleCandidates: [],
    seniority: 'senior',
    targetRegions: ['ua'] as const,
    workModes: ['remote'] as const,
    allowedSources: ['djinni'],
    profileSkills: ['react', 'typescript'],
    profileKeywords: ['frontend'],
    searchTerms: ['react'],
    excludeTerms: ['wordpress'],
  },
};

describe('search profile build state', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'window', {
      value: { localStorage: createStorage() },
      configurable: true,
    });
  });

  afterEach(() => {
    // @ts-expect-error test-only cleanup
    delete globalThis.window;
  });

  it('restores a build for the same profile and inputs', () => {
    writeStoredSearchProfileBuild({
      profileId: 'profile-a',
      rawText: 'React engineer',
      preferences,
      result: buildResult,
    });

    expect(
      readStoredSearchProfileBuild({
        profileId: 'profile-a',
        rawText: 'React engineer',
        preferences,
      }),
    ).toEqual(buildResult);
  });

  it('falls back to an anonymous build when the profile is later persisted', () => {
    writeStoredSearchProfileBuild({
      profileId: null,
      rawText: 'React engineer',
      preferences,
      result: buildResult,
    });

    expect(
      readStoredSearchProfileBuild({
        profileId: 'profile-a',
        rawText: 'React engineer',
        preferences,
      }),
    ).toEqual(buildResult);
  });
});
