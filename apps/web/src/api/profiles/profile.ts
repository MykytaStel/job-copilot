import type { CandidateProfile, CandidateProfileInput } from '@job-copilot/shared/profiles';
import { json, readStoredProfileId, request, writeStoredProfileId } from '../client';
import type { EngineAnalyzeProfile, EngineProfile, EngineResume } from '../engine-types';
import { mapProfile } from '../mappers';
import type { PersistedSearchPreferences } from './types';

export type PersistedCandidateProfile = CandidateProfile & {
  searchPreferences?: PersistedSearchPreferences;
};

function mapPersistedProfile(profile: EngineProfile): PersistedCandidateProfile {
  return {
    ...mapProfile(profile),
    searchPreferences: profile.search_preferences
      ? {
          targetRegions: profile.search_preferences.target_regions,
          workModes: profile.search_preferences.work_modes,
          preferredRoles: profile.search_preferences.preferred_roles,
          allowedSources: profile.search_preferences.allowed_sources,
          includeKeywords: profile.search_preferences.include_keywords,
          excludeKeywords: profile.search_preferences.exclude_keywords,
        }
      : undefined,
  };
}

export async function getStoredProfileRawText(): Promise<string> {
  const profileId = readStoredProfileId();
  if (!profileId) return '';
  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return profile.raw_text;
}

export async function analyzeStoredProfile(): Promise<EngineAnalyzeProfile> {
  const profileId = readStoredProfileId();
  if (!profileId) {
    throw new Error('Create a profile first');
  }
  return request<EngineAnalyzeProfile>(`/api/v1/profiles/${profileId}/analyze`, json('POST', {}));
}

export async function getProfile(): Promise<PersistedCandidateProfile | undefined> {
  const profileId = readStoredProfileId();
  if (!profileId) return undefined;
  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return mapPersistedProfile(profile);
}

export async function saveProfile(
  payload: CandidateProfileInput & { rawText: string; searchPreferences?: PersistedSearchPreferences },
): Promise<PersistedCandidateProfile> {
  const profileId = readStoredProfileId();

  const body = {
    name: payload.name,
    email: payload.email,
    location: payload.location ?? null,
    raw_text: payload.rawText,
    years_of_experience: payload.yearsOfExperience ?? null,
    salary_min: payload.salaryMin ?? null,
    salary_max: payload.salaryMax ?? null,
    salary_currency: payload.salaryCurrency ?? null,
    languages: payload.languages,
    search_preferences: payload.searchPreferences
      ? {
          target_regions: payload.searchPreferences.targetRegions,
          work_modes: payload.searchPreferences.workModes,
          preferred_roles: payload.searchPreferences.preferredRoles,
          allowed_sources: payload.searchPreferences.allowedSources,
          include_keywords: payload.searchPreferences.includeKeywords,
          exclude_keywords: payload.searchPreferences.excludeKeywords,
        }
      : null,
  };

  const profile = profileId
    ? await request<EngineProfile>(`/api/v1/profiles/${profileId}`, json('PATCH', body))
    : await request<EngineProfile>('/api/v1/profiles', json('POST', body));

  writeStoredProfileId(profile.id);

  // Upload resume in parallel with analysis so the engine-api fit-score endpoint
  // has an active resume to work with (resumes and profiles are separate records).
  const [analyzed] = await Promise.all([
    request<EngineAnalyzeProfile>(`/api/v1/profiles/${profile.id}/analyze`, json('POST', {})),
    request<EngineResume>(
      '/api/v1/resume/upload',
      json('POST', {
        filename: 'profile.md',
        raw_text: payload.rawText,
      }),
    ).catch((err: unknown) => {
      console.warn('Resume upload failed during profile save; profile was still created:', err);
      return null;
    }),
  ]);

  return {
    ...mapPersistedProfile(profile),
    summary: analyzed.summary,
    skills: analyzed.skills,
  };
}

export async function saveProfileSearchPreferences(
  profileId: string,
  searchPreferences: PersistedSearchPreferences,
): Promise<PersistedCandidateProfile> {
  const profile = await request<EngineProfile>(
    `/api/v1/profiles/${profileId}`,
    json('PATCH', {
      search_preferences: {
        target_regions: searchPreferences.targetRegions,
        work_modes: searchPreferences.workModes,
        preferred_roles: searchPreferences.preferredRoles,
        allowed_sources: searchPreferences.allowedSources,
        include_keywords: searchPreferences.includeKeywords,
        exclude_keywords: searchPreferences.excludeKeywords,
      },
    }),
  );

  return mapPersistedProfile(profile);
}
