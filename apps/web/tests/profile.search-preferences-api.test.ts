import { afterEach, describe, expect, it, vi } from 'vitest';

import { saveProfileSearchPreferences } from '../src/api/profiles';

describe('saveProfileSearchPreferences', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('patches persisted search preferences and maps the updated profile response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        id: 'profile_123',
        name: 'Jane Doe',
        email: 'jane@example.com',
        location: 'Kyiv',
        raw_text: 'Senior frontend engineer',
        years_of_experience: 6,
        salary_min: 4000,
        salary_max: 5500,
        salary_currency: 'USD',
        languages: ['English'],
        search_preferences: {
          target_regions: ['ua', 'eu_remote'],
          work_modes: ['remote'],
          preferred_roles: ['frontend_engineer'],
          allowed_sources: ['djinni', 'work_ua'],
          include_keywords: ['product company'],
          exclude_keywords: ['gambling'],
        },
        analysis: {
          summary: 'Experienced frontend engineer',
          primary_role: 'frontend_engineer',
          seniority: 'senior',
          skills: ['React', 'TypeScript'],
          keywords: ['frontend'],
        },
        created_at: '2026-04-20T00:00:00Z',
        updated_at: '2026-04-22T00:00:00Z',
        skills_updated_at: '2026-04-22T00:00:00Z',
      }),
    });

    vi.stubGlobal('fetch', fetchMock);

    const profile = await saveProfileSearchPreferences('profile_123', {
      targetRegions: ['ua', 'eu_remote'],
      workModes: ['remote'],
      preferredRoles: ['frontend_engineer'],
      allowedSources: ['djinni', 'work_ua'],
      includeKeywords: ['product company'],
      excludeKeywords: ['gambling'],
			scoringWeights: {
				skillMatchImportance: 8,
				salaryFitImportance: 6,
				jobFreshnessImportance: 5,
				remoteWorkImportance: 5,
			},
    });

    expect(fetchMock.mock.calls[0]?.[0]).toBeDefined();
    expect(String(fetchMock.mock.calls[0]![0])).toContain('/api/v1/profiles/profile_123');
    expect(fetchMock.mock.calls[0]?.[1]).toEqual(
      expect.objectContaining({
        method: 'PATCH',
      }),
    );
    expect(JSON.parse(fetchMock.mock.calls[0]![1]!.body as string)).toEqual({
      search_preferences: {
        target_regions: ['ua', 'eu_remote'],
        work_modes: ['remote'],
        preferred_roles: ['frontend_engineer'],
        allowed_sources: ['djinni', 'work_ua'],
        include_keywords: ['product company'],
        exclude_keywords: ['gambling'],
      },
    });
    expect(profile.searchPreferences).toEqual({
      targetRegions: ['ua', 'eu_remote'],
      workModes: ['remote'],
      preferredRoles: ['frontend_engineer'],
      allowedSources: ['djinni', 'work_ua'],
      includeKeywords: ['product company'],
      excludeKeywords: ['gambling'],
    });
    expect(profile.summary).toBe('Experienced frontend engineer');
    expect(profile.skills).toEqual(['React', 'TypeScript']);
  });
});
