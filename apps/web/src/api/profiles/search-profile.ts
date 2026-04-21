import { json, request } from '../client';
import type { EngineBuildSearchProfileResponse } from '../engine-types';
import { normalizeMissingString } from '../mappers';
import type { SearchProfileBuildRequest, SearchProfileBuildResult } from './types';

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
