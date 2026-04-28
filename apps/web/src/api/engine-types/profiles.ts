import type {
  EngineAnalyzeProfileResponse as SharedEngineAnalyzeProfileResponse,
  EngineRoleCandidateResponse,
} from '@job-copilot/shared/profiles';

export type EngineResume = {
  id: string;
  version: number;
  filename: string;
  raw_text: string;
  is_active: boolean;
  uploaded_at: string;
};

export type EngineMatchResult = {
  id: string;
  job_id: string;
  resume_id: string;
  score: number;
  matched_skills: string[];
  missing_skills: string[];
  notes: string;
  created_at: string;
};

export type EngineRoleCandidate = EngineRoleCandidateResponse;

export type EngineAnalyzeProfile = Omit<
  SharedEngineAnalyzeProfileResponse,
  'role_candidates' | 'suggested_search_terms'
> & {
  role_candidates?: EngineRoleCandidate[];
  suggested_search_terms?: string[];
};

export type EngineProfileAnalysis = Pick<
  SharedEngineAnalyzeProfileResponse,
  'summary' | 'primary_role' | 'seniority' | 'skills' | 'keywords'
>;

export type EngineProfile = {
  id: string;
  name: string;
  email: string;
  location?: string | null;
  raw_text: string;
  years_of_experience?: number | null;
  salary_min?: number | null;
  salary_max?: number | null;
  salary_currency?: string | null;
  languages?: string[] | null;
  search_preferences?: {
    target_regions: (
      | 'ua'
      | 'eu'
      | 'eu_remote'
      | 'poland'
      | 'germany'
      | 'uk'
      | 'us'
    )[];
    work_modes: ('remote' | 'hybrid' | 'onsite')[];
    preferred_roles: string[];
    allowed_sources: string[];
    include_keywords: string[];
    exclude_keywords: string[];
		scoring_weights?: EngineProfileScoringWeights | null;
  } | null;
  analysis?: EngineProfileAnalysis | null;
  created_at: string;
  updated_at: string;
  skills_updated_at?: string | null;
};

export type EngineProfileScoringWeights = {
  skill_match_importance: number;
  salary_fit_importance: number;
  job_freshness_importance: number;
  remote_work_importance: number;
};