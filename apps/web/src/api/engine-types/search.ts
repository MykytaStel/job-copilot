import type { EngineGlobalSearchApplication } from './applications';
import type { EngineJob } from './jobs';
import type { EngineAnalyzeProfile } from './profiles';

export type InternalSearchTargetRegion =
  | 'ua'
  | 'eu'
  | 'eu_remote'
  | 'poland'
  | 'germany'
  | 'uk'
  | 'us';

export type InternalSearchWorkMode = 'remote' | 'hybrid' | 'onsite';

export type EngineSearchRoleCandidate = {
  role: string;
  confidence: number;
};

export type EngineSearchProfile = {
  primary_role: string;
  primary_role_confidence?: number | null;
  target_roles: string[];
  role_candidates: EngineSearchRoleCandidate[];
  seniority: string;
  target_regions: InternalSearchTargetRegion[];
  work_modes: InternalSearchWorkMode[];
  allowed_sources: string[];
  profile_skills: string[];
  profile_keywords: string[];
  search_terms: string[];
  exclude_terms: string[];
};

export type EngineBuildSearchProfileResponse = {
  analyzed_profile: EngineAnalyzeProfile;
  search_profile: EngineSearchProfile;
};

export type EngineFitExplanation = {
  job_id: string;
  score: number;
  matched_roles: string[];
  matched_skills: string[];
  matched_keywords: string[];
  missing_signals: string[];
  source_match: boolean;
  work_mode_match?: boolean | null;
  region_match?: boolean | null;
  description_quality: string;
  positive_reasons: string[];
  negative_reasons: string[];
  reasons: string[];
};

export type EngineRankedJobResult = {
  job: EngineJob;
  fit: EngineFitExplanation;
};

export type EngineSearchRunMeta = {
  total_candidates: number;
  filtered_out_by_source: number;
  filtered_out_hidden: number;
  filtered_out_company_blacklist: number;
  scored_jobs: number;
  returned_jobs: number;
  low_evidence_jobs: number;
  weak_description_jobs: number;
  role_mismatch_jobs: number;
  seniority_mismatch_jobs: number;
  source_mismatch_jobs: number;
  top_missing_signals: string[];
};

export type EngineRunSearchResponse = {
  results: EngineRankedJobResult[];
  meta: EngineSearchRunMeta;
};

export type EngineGlobalSearchResponse = {
  jobs: EngineJob[];
  applications: EngineGlobalSearchApplication[];
};