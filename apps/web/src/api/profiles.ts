import type {
  CandidateProfile,
  CandidateProfileInput,
  ResumeUploadInput,
  ResumeVersion,
} from '@job-copilot/shared/profiles';
import { json, readStoredProfileId, request, writeStoredProfileId } from './client';
import type {
  EngineAnalyzeProfile,
  EngineBuildSearchProfileResponse,
  EngineProfile,
  EngineResume,
  EngineRoleCandidate,
  EngineRoleCatalogResponse,
  EngineSearchRoleCandidate,
  EngineSourceCatalogResponse,
} from './engine-types';
import { mapProfile, mapResume, normalizeMissingString } from './mappers';

export type SourceCatalogItem = {
  id: string;
  displayName: string;
};

export type RoleCatalogItem = {
  id: string;
  displayName: string;
  family?: string;
  deprecatedApiIds: string[];
  isFallback: boolean;
};

export type SearchTargetRegion = 'ua' | 'eu' | 'eu_remote' | 'poland' | 'germany' | 'uk' | 'us';

export type SearchWorkMode = 'remote' | 'hybrid' | 'onsite';

export type SearchProfileBuildRequest = {
  rawText: string;
  preferences?: {
    targetRegions?: SearchTargetRegion[];
    workModes?: SearchWorkMode[];
    preferredRoles?: string[];
    allowedSources?: string[];
    includeKeywords?: string[];
    excludeKeywords?: string[];
  };
};

export type SearchProfileBuildResult = {
  analyzedProfile: {
    summary: string;
    primaryRole: string;
    seniority: string;
    skills: string[];
    keywords: string[];
    roleCandidates: EngineRoleCandidate[];
    suggestedSearchTerms: string[];
  };
  searchProfile: {
    primaryRole: string;
    primaryRoleConfidence?: number;
    targetRoles: string[];
    roleCandidates: EngineSearchRoleCandidate[];
    seniority: string;
    targetRegions: SearchTargetRegion[];
    workModes: SearchWorkMode[];
    allowedSources: string[];
    profileSkills: string[];
    profileKeywords: string[];
    searchTerms: string[];
    excludeTerms: string[];
  };
};

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

export async function getProfile(): Promise<CandidateProfile | undefined> {
  const profileId = readStoredProfileId();
  if (!profileId) return undefined;

  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return mapProfile(profile);
}

export async function saveProfile(
  payload: CandidateProfileInput & { rawText: string },
): Promise<CandidateProfile> {
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
    ).catch(() => null),
  ]);

  return {
    ...mapProfile(profile),
    summary: analyzed.summary,
    skills: analyzed.skills,
  };
}

export async function getResumes(): Promise<ResumeVersion[]> {
  const resumes = await request<EngineResume[]>('/api/v1/resumes');
  return resumes.map(mapResume);
}

export async function getActiveResume(): Promise<ResumeVersion> {
  const resume = await request<EngineResume>('/api/v1/resumes/active');
  return mapResume(resume);
}

export async function uploadResume(payload: ResumeUploadInput): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    '/api/v1/resume/upload',
    json('POST', {
      filename: payload.filename,
      raw_text: payload.rawText,
    }),
  );
  return mapResume(resume);
}

export async function activateResume(id: string): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(`/api/v1/resumes/${id}/activate`, json('POST', {}));
  return mapResume(resume);
}

export async function getSources(): Promise<SourceCatalogItem[]> {
  const response = await request<EngineSourceCatalogResponse>('/api/v1/sources');
  return response.sources.map((source) => ({
    id: source.id,
    displayName: source.display_name,
  }));
}

export async function getRoles(): Promise<RoleCatalogItem[]> {
  const response = await request<EngineRoleCatalogResponse>('/api/v1/roles');
  return response.roles.map((role) => ({
    id: role.id,
    displayName: role.display_name,
    family: role.family,
    deprecatedApiIds: role.deprecated_api_ids,
    isFallback: role.is_fallback,
  }));
}

export async function buildSearchProfile(
  payload: SearchProfileBuildRequest,
): Promise<SearchProfileBuildResult> {
  const response = await request<EngineBuildSearchProfileResponse>(
    '/api/v1/search-profile/build',
    json('POST', {
      raw_text: payload.rawText,
      preferences: {
        target_regions: payload.preferences?.targetRegions ?? [],
        work_modes: payload.preferences?.workModes ?? [],
        preferred_roles: payload.preferences?.preferredRoles ?? [],
        allowed_sources: payload.preferences?.allowedSources ?? [],
        include_keywords: payload.preferences?.includeKeywords ?? [],
        exclude_keywords: payload.preferences?.excludeKeywords ?? [],
      },
    }),
  );

  return {
    analyzedProfile: {
      summary: response.analyzed_profile.summary,
      primaryRole: response.analyzed_profile.primary_role,
      seniority: normalizeMissingString(response.analyzed_profile.seniority) ?? '',
      skills: response.analyzed_profile.skills,
      keywords: response.analyzed_profile.keywords,
      roleCandidates: response.analyzed_profile.role_candidates ?? [],
      suggestedSearchTerms: response.analyzed_profile.suggested_search_terms ?? [],
    },
    searchProfile: {
      primaryRole: response.search_profile.primary_role,
      primaryRoleConfidence: response.search_profile.primary_role_confidence ?? undefined,
      targetRoles: response.search_profile.target_roles,
      roleCandidates: response.search_profile.role_candidates ?? [],
      seniority: normalizeMissingString(response.search_profile.seniority) ?? '',
      targetRegions: response.search_profile.target_regions,
      workModes: response.search_profile.work_modes,
      allowedSources: response.search_profile.allowed_sources,
      profileSkills: response.search_profile.profile_skills ?? [],
      profileKeywords: response.search_profile.profile_keywords ?? [],
      searchTerms: response.search_profile.search_terms,
      excludeTerms: response.search_profile.exclude_terms,
    },
  };
}
