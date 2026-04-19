import type {
  EngineBuildSearchProfileResponse as SharedEngineBuildSearchProfileResponse,
  EngineRoleId,
  EngineSearchProfileResponse as SharedEngineSearchProfileResponse,
  EngineTargetRegion,
  EngineWorkMode,
} from '@job-copilot/shared';

import type { EngineGlobalSearchApplication } from './applications';
import type { EngineJob } from './jobs';
import type { EngineAnalyzeProfile } from './profiles';

export type InternalSearchTargetRegion = EngineTargetRegion;

export type InternalSearchWorkMode = EngineWorkMode;

export type EngineSearchRoleCandidate = {
  role: EngineRoleId;
  confidence: number;
};

export type EngineSearchProfile = Omit<SharedEngineSearchProfileResponse, 'role_candidates'> & {
  primary_role_confidence?: number | null;
  role_candidates: EngineSearchRoleCandidate[];
  profile_skills?: string[] | null;
  profile_keywords?: string[] | null;
};

export type EngineBuildSearchProfileResponse = Omit<
  SharedEngineBuildSearchProfileResponse,
  'analyzed_profile' | 'search_profile'
> & {
  analyzed_profile: EngineAnalyzeProfile;
  search_profile: EngineSearchProfile;
};

export type EngineFitExplanation = {
  job_id: string;
  score: number;
  score_breakdown?: EngineScoreBreakdown;
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

export type EngineScoreBreakdown = {
  total_score: number;
  matching_score: number;
  salary_score: number;
  reranker_score: number;
  freshness_score: number;
  penalties: EngineScorePenalty[];
  reranker_mode: 'deterministic' | 'learned' | 'trained' | 'fallback';
};

export type EngineScorePenalty = {
  kind: string;
  score_delta: number;
  reason: string;
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
