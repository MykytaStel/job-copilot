import type { EngineRoleCandidate, EngineSearchRoleCandidate } from '../engine-types';

export type SourceCatalogItem = {
  id: string;
  displayName: string;
};

export type RoleCatalogItem = {
  id: string;
  displayName: string;
  family?: string;
  isFallback: boolean;
};

export type SearchTargetRegion = 'ua' | 'eu' | 'eu_remote' | 'poland' | 'germany' | 'uk' | 'us';

export type SearchWorkMode = 'remote' | 'hybrid' | 'onsite';

export type PersistedSearchPreferences = {
  targetRegions: SearchTargetRegion[];
  workModes: SearchWorkMode[];
  preferredRoles: string[];
  allowedSources: string[];
  includeKeywords: string[];
  excludeKeywords: string[];
};

export type SearchProfileBuildRequest = {
  rawText: string;
  preferences?: PersistedSearchPreferences;
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
