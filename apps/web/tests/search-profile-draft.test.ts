import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
  clearSearchProfileDraft,
  readSearchProfileDraft,
  writeSearchProfileDraft,
} from '../src/features/profile/searchProfileDraft';

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

describe('search profile draft storage', () => {
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

  it('persists and restores the current draft', () => {
    writeSearchProfileDraft({
      targetRegions: ['ukraine'],
      workModes: ['remote'],
      preferredRoles: ['frontend_engineer'],
      allowedSources: ['djinni'],
      includeKeywordsInput: 'react, typescript',
      excludeKeywordsInput: 'wordpress',
    });

    expect(readSearchProfileDraft()).toEqual({
      targetRegions: ['ukraine'],
      workModes: ['remote'],
      preferredRoles: ['frontend_engineer'],
      allowedSources: ['djinni'],
      includeKeywordsInput: 'react, typescript',
      excludeKeywordsInput: 'wordpress',
    });
  });

  it('returns null for invalid stored payloads and clears cleanly', () => {
    window.localStorage.setItem('job-copilot.search-profile-draft.v1', '{bad json');

    expect(readSearchProfileDraft()).toBeNull();

    clearSearchProfileDraft();
    expect(readSearchProfileDraft()).toBeNull();
  });

  it('keeps drafts scoped to the active profile when a profile id is provided', () => {
    writeSearchProfileDraft(
      {
        targetRegions: ['ua'],
        workModes: ['remote'],
        preferredRoles: ['frontend_engineer'],
        allowedSources: ['djinni'],
        includeKeywordsInput: 'react',
        excludeKeywordsInput: '',
      },
      'profile-a',
    );
    writeSearchProfileDraft(
      {
        targetRegions: ['eu'],
        workModes: ['hybrid'],
        preferredRoles: ['backend_engineer'],
        allowedSources: ['workua'],
        includeKeywordsInput: 'rust',
        excludeKeywordsInput: 'php',
      },
      'profile-b',
    );

    expect(readSearchProfileDraft('profile-a')?.preferredRoles).toEqual(['frontend_engineer']);
    expect(readSearchProfileDraft('profile-b')?.preferredRoles).toEqual(['backend_engineer']);

    clearSearchProfileDraft('profile-a');
    expect(readSearchProfileDraft('profile-a')).toBeNull();
    expect(readSearchProfileDraft('profile-b')?.preferredRoles).toEqual(['backend_engineer']);
  });
});
