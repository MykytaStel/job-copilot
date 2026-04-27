import { describe, expect, it } from 'vitest';

import {
  buildPersistedSearchPreferences,
  buildSearchProfileDraftFromPreferences,
  resolveSearchProfileDraft,
} from '../src/features/profile/searchProfilePreferences';

describe('search profile preference helpers', () => {
  it('converts persisted preferences into builder draft inputs', () => {
    expect(
      buildSearchProfileDraftFromPreferences({
        targetRegions: ['ua', 'eu_remote'],
        workModes: ['remote'],
        preferredRoles: ['frontend_engineer'],
        allowedSources: ['djinni', 'work_ua'],
        includeKeywords: ['product company', 'typescript'],
        excludeKeywords: ['gambling'],
      }),
    ).toEqual({
      targetRegions: ['ua', 'eu_remote'],
      workModes: ['remote'],
      preferredRoles: ['frontend_engineer'],
      allowedSources: ['djinni', 'work_ua'],
      includeKeywordsInput: 'product company, typescript',
      excludeKeywordsInput: 'gambling',
    });
  });

  it('normalizes builder draft inputs before persistence', () => {
    expect(
			buildPersistedSearchPreferences({
				targetRegions: ['ua'],
				workModes: ['remote'],
				preferredRoles: ['frontend_engineer'],
				allowedSources: ['djinni', 'work_ua'],
				includeKeywordsInput: 'react, typescript',
				excludeKeywordsInput: 'gambling\n outsourcing ',
			}),
		).toEqual({
			targetRegions: ['ua'],
			workModes: ['remote'],
			preferredRoles: ['frontend_engineer'],
			allowedSources: ['djinni', 'work_ua'],
			includeKeywords: ['react', 'typescript'],
			excludeKeywords: ['gambling', 'outsourcing'],
			scoringWeights: {
				skillMatchImportance: 8,
				salaryFitImportance: 6,
				jobFreshnessImportance: 5,
				remoteWorkImportance: 5,
			},
		});
  });

  it('prefers persisted preferences over local draft fallback', () => {
    expect(
      resolveSearchProfileDraft(
        {
          targetRegions: ['eu'],
          workModes: ['hybrid'],
          preferredRoles: ['backend_engineer'],
          allowedSources: ['dou_ua'],
          includeKeywords: ['rust'],
          excludeKeywords: [],
        },
        {
          targetRegions: ['ua'],
          workModes: ['remote'],
          preferredRoles: ['frontend_engineer'],
          allowedSources: ['djinni'],
          includeKeywordsInput: 'react',
          excludeKeywordsInput: 'gambling',
        },
      ),
    ).toEqual({
      targetRegions: ['eu'],
      workModes: ['hybrid'],
      preferredRoles: ['backend_engineer'],
      allowedSources: ['dou_ua'],
      includeKeywordsInput: 'rust',
      excludeKeywordsInput: '',
    });
  });
});
