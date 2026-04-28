import type { CandidateProfile, CandidateProfileInput } from '@job-copilot/shared/profiles';
import { json, readStoredProfileId, request, writeStoredProfileId } from '../client';
import type { EngineAnalyzeProfile, EngineProfile, EngineResume } from '../engine-types';
import { mapProfile } from '../mappers';
import {
  DEFAULT_SCORING_WEIGHTS,
  type PersistedSearchPreferences,
  type ScoringWeights,
} from './types';

export type PersistedCandidateProfile = CandidateProfile & {
  searchPreferences?: PersistedSearchPreferences;
};

function clampScoringWeight(value: number | undefined, fallback: number): number {
  if (typeof value !== 'number' || Number.isNaN(value)) return fallback;

  return Math.min(10, Math.max(1, Math.round(value)));
}

export function mapScoringWeights(
  weights?: {
    skill_match_importance?: number;
    salary_fit_importance?: number;
    job_freshness_importance?: number;
    remote_work_importance?: number;
  } | null,
): ScoringWeights {
  return {
    skillMatchImportance: clampScoringWeight(
      weights?.skill_match_importance,
      DEFAULT_SCORING_WEIGHTS.skillMatchImportance,
    ),
    salaryFitImportance: clampScoringWeight(
      weights?.salary_fit_importance,
      DEFAULT_SCORING_WEIGHTS.salaryFitImportance,
    ),
    jobFreshnessImportance: clampScoringWeight(
      weights?.job_freshness_importance,
      DEFAULT_SCORING_WEIGHTS.jobFreshnessImportance,
    ),
    remoteWorkImportance: clampScoringWeight(
      weights?.remote_work_importance,
      DEFAULT_SCORING_WEIGHTS.remoteWorkImportance,
    ),
  };
}

function toEngineScoringWeights(
  weights: ScoringWeights = DEFAULT_SCORING_WEIGHTS,
) {
  return {
    skill_match_importance: weights.skillMatchImportance,
    salary_fit_importance: weights.salaryFitImportance,
    job_freshness_importance: weights.jobFreshnessImportance,
    remote_work_importance: weights.remoteWorkImportance,
  };
}

function toEngineSearchPreferences(searchPreferences: PersistedSearchPreferences) {
  return {
    target_regions: searchPreferences.targetRegions,
    work_modes: searchPreferences.workModes,
    preferred_roles: searchPreferences.preferredRoles,
    allowed_sources: searchPreferences.allowedSources,
    include_keywords: searchPreferences.includeKeywords,
    exclude_keywords: searchPreferences.excludeKeywords,
    scoring_weights: toEngineScoringWeights(
      searchPreferences.scoringWeights ?? DEFAULT_SCORING_WEIGHTS,
    ),
  };
}

function mapSearchPreferences(
  preferences: EngineProfile['search_preferences'],
): PersistedSearchPreferences | undefined {
  if (!preferences) return undefined;

  return {
    targetRegions: preferences.target_regions,
    workModes: preferences.work_modes,
    preferredRoles: preferences.preferred_roles,
    allowedSources: preferences.allowed_sources,
    includeKeywords: preferences.include_keywords,
    excludeKeywords: preferences.exclude_keywords,
    scoringWeights: mapScoringWeights(preferences.scoring_weights),
  };
}

function mapPersistedProfile(profile: EngineProfile): PersistedCandidateProfile {
  return {
    ...mapProfile(profile),
    searchPreferences: mapSearchPreferences(profile.search_preferences),
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
  payload: CandidateProfileInput & {
    rawText: string;
    searchPreferences?: PersistedSearchPreferences;
  },
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
    preferred_locations: payload.preferredLocations,
    work_mode_preference: payload.workModePreference ?? 'any',
    search_preferences: payload.searchPreferences
      ? toEngineSearchPreferences(payload.searchPreferences)
      : null,
		portfolio_url: payload.portfolioUrl ?? null,
		github_url: payload.githubUrl ?? null,
		linkedin_url: payload.linkedinUrl ?? null,
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
  };
}

export async function updateProfileSkills(
  profileId: string,
  skills: string[],
): Promise<PersistedCandidateProfile> {
  const profile = await request<EngineProfile>(
    `/api/v1/profiles/${profileId}`,
    json('PATCH', { skills }),
  );

  return mapPersistedProfile(profile);
}

export async function saveProfileSearchPreferences(
  profileId: string,
  searchPreferences: PersistedSearchPreferences,
): Promise<PersistedCandidateProfile> {
  const profile = await request<EngineProfile>(
    `/api/v1/profiles/${profileId}`,
    json('PATCH', {
  		search_preferences: toEngineSearchPreferences(searchPreferences),
		}),
  );

  return mapPersistedProfile(profile);
}


export async function updateScoringWeights(
  weights: ScoringWeights,
): Promise<PersistedCandidateProfile> {
  const profileId = readStoredProfileId();

  if (!profileId) {
    throw new Error('Create a profile first');
  }

  const current = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  const currentPreferences =
    mapSearchPreferences(current.search_preferences) ?? {
      targetRegions: [],
      workModes: [],
      preferredRoles: [],
      allowedSources: [],
      includeKeywords: [],
      excludeKeywords: [],
      scoringWeights: DEFAULT_SCORING_WEIGHTS,
    };

  const updated = await request<EngineProfile>(
    `/api/v1/profiles/${profileId}`,
    json('PATCH', {
      search_preferences: toEngineSearchPreferences({
        ...currentPreferences,
        scoringWeights: weights,
      }),
    }),
  );

  return mapPersistedProfile(updated);
}
