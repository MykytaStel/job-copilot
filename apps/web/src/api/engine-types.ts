import type {
  ApplicationStatus,
  CompanyFeedbackStatus,
} from '@job-copilot/shared';

type InternalAppNotificationType =
  | 'new_jobs_found'
  | 'job_reactivated'
  | 'application_due_soon';

type InternalMarketTrend = 'up' | 'down' | 'stable';
type InternalSearchTargetRegion =
  | 'ua'
  | 'eu'
  | 'eu_remote'
  | 'poland'
  | 'germany'
  | 'uk'
  | 'us';
type InternalSearchWorkMode = 'remote' | 'hybrid' | 'onsite';

type EngineApiError = {
  code?: string;
  message?: string;
  details?: unknown;
};

type EngineHealthResponse = {
  status: string;
  database: {
    status: string;
    configured: boolean;
    migrations_enabled_on_startup: boolean;
  };
};

type EngineRecentJobsResponse = {
  jobs: EngineJob[];
  summary: EngineJobFeedSummary;
};

type EngineSourceCatalogResponse = {
  sources: EngineSourceCatalogItem[];
};

type EngineRoleCatalogResponse = {
  roles: EngineRoleCatalogItem[];
};

type EngineSourceCatalogItem = {
  id: string;
  display_name: string;
};

type EngineRoleCatalogItem = {
  id: string;
  display_name: string;
  deprecated_api_ids: string[];
  family?: string;
  is_fallback: boolean;
};

type EngineRecentApplicationsResponse = {
  applications: EngineApplication[];
};

type EngineFeedbackOverviewResponse = {
  profile_id: string;
  jobs: EngineJobFeedbackRecord[];
  companies: EngineCompanyFeedbackRecord[];
  summary: {
    saved_jobs_count: number;
    hidden_jobs_count: number;
    bad_fit_jobs_count: number;
    whitelisted_companies_count: number;
    blacklisted_companies_count: number;
  };
};

type EngineMarketOverview = {
  new_jobs_this_week: number;
  active_companies_count: number;
  active_jobs_count: number;
  remote_percentage: number;
};

type EngineMarketCompanyEntry = {
  company_name: string;
  active_jobs: number;
  this_week: number;
  prev_week: number;
  velocity: number;
};

type EngineMarketCompaniesResponse = {
  companies: EngineMarketCompanyEntry[];
};

type EngineMarketSalaryTrend = {
  seniority: string;
  p25: number;
  median: number;
  p75: number;
  sample_count: number;
};

type EngineMarketRoleDemandEntry = {
  role_group: string;
  this_period: number;
  prev_period: number;
  trend: InternalMarketTrend;
};

type EngineNotification = {
  id: string;
  profile_id: string;
  type: InternalAppNotificationType;
  title: string;
  body?: string | null;
  payload?: Record<string, unknown> | null;
  read_at?: string | null;
  created_at: string;
};

type EngineNotificationsResponse = {
  notifications: EngineNotification[];
};

type EngineUnreadNotificationsCountResponse = {
  profile_id: string;
  unread_count: number;
};

type EngineContactsResponse = {
  contacts: EngineContact[];
};

type EngineResume = {
  id: string;
  version: number;
  filename: string;
  raw_text: string;
  is_active: boolean;
  uploaded_at: string;
};

type EngineMatchResult = {
  id: string;
  job_id: string;
  resume_id: string;
  score: number;
  matched_skills: string[];
  missing_skills: string[];
  notes: string;
  created_at: string;
};

type EngineJob = {
  id: string;
  title: string;
  company_name: string;
  description_text: string;
  location?: string | null;
  remote_type?: string | null;
  seniority?: string | null;
  salary_min?: number | null;
  salary_max?: number | null;
  salary_currency?: string | null;
  posted_at: string | null;
  first_seen_at: string;
  last_seen_at: string;
  is_active: boolean;
  inactivated_at?: string | null;
  reactivated_at?: string | null;
  lifecycle_stage: 'active' | 'inactive' | 'reactivated';
  primary_variant?: {
    source: string;
    source_job_id: string;
    source_url: string;
    fetched_at: string;
    last_seen_at: string;
    is_active: boolean;
    inactivated_at?: string | null;
  } | null;
  presentation: {
    title: string;
    company: string;
    summary?: string | null;
    summary_quality?: string | null;
    summary_fallback: boolean;
    description_quality: string;
    location_label?: string | null;
    work_mode_label?: string | null;
    source_label?: string | null;
    outbound_url?: string | null;
    salary_label?: string | null;
    freshness_label?: string | null;
    badges: string[];
  };
  feedback: EngineJobFeedbackState;
};

type EngineJobFeedbackState = {
  saved: boolean;
  hidden: boolean;
  bad_fit: boolean;
  company_status?: CompanyFeedbackStatus | null;
};

type EngineJobFeedbackRecord = {
  job_id: string;
  saved: boolean;
  hidden: boolean;
  bad_fit: boolean;
  updated_at: string;
};

type EngineCompanyFeedbackRecord = {
  company_name: string;
  normalized_company_name: string;
  status: CompanyFeedbackStatus;
  updated_at: string;
};

type EngineJobFeedSummary = {
  total_jobs: number;
  active_jobs: number;
  inactive_jobs: number;
  reactivated_jobs: number;
};

type EngineApplication = {
  id: string;
  job_id: string;
  resume_id: string | null;
  status: ApplicationStatus;
  applied_at: string | null;
  due_date: string | null;
  updated_at: string;
};

type EngineGlobalSearchApplication = {
  id: string;
  job_id: string;
  status: ApplicationStatus;
  applied_at: string | null;
  due_date: string | null;
  updated_at: string;
  job_title: string;
  company_name: string;
};

type EngineGlobalSearchResponse = {
  jobs: EngineJob[];
  applications: EngineGlobalSearchApplication[];
};

type EngineApplicationDetail = EngineApplication & {
  job: EngineJob;
  resume: EngineResume | null;
  offer?: EngineOffer | null;
  notes: Array<{
    id: string;
    application_id: string;
    content: string;
    created_at: string;
  }>;
  contacts: Array<{
    id: string;
    application_id: string;
    relationship: string;
    contact: {
      id: string;
      name: string;
      email?: string | null;
      phone?: string | null;
      linkedin_url?: string | null;
      company?: string | null;
      role?: string | null;
      created_at: string;
    };
  }>;
  activities: Array<{
    id: string;
    application_id: string;
    activity_type: string;
    description: string;
    happened_at: string;
    created_at: string;
  }>;
  tasks: Array<{
    id: string;
    application_id: string;
    title: string;
    remind_at?: string | null;
    done: boolean;
    created_at: string;
  }>;
};

type EngineContact = {
  id: string;
  name: string;
  email?: string | null;
  phone?: string | null;
  linkedin_url?: string | null;
  company?: string | null;
  role?: string | null;
  created_at: string;
};

type EngineOffer = {
  id: string;
  application_id: string;
  status: string;
  compensation_min?: number | null;
  compensation_max?: number | null;
  compensation_currency?: string | null;
  starts_at?: string | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
};

type EngineProfile = {
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
  analysis?: {
    summary: string;
    primary_role: string;
    seniority: string;
    skills: string[];
    keywords: string[];
  } | null;
  created_at: string;
  updated_at: string;
  skills_updated_at?: string | null;
};

type EngineAnalyzeProfile = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
  role_candidates?: EngineRoleCandidate[];
  suggested_search_terms?: string[];
};

type EngineRoleCandidate = {
  role: string;
  score: number;
  confidence: number;
  matched_signals: string[];
};

type EngineSearchRoleCandidate = {
  role: string;
  confidence: number;
};

type EngineSearchProfile = {
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

type EngineBuildSearchProfileResponse = {
  analyzed_profile: EngineAnalyzeProfile;
  search_profile: EngineSearchProfile;
};

type EngineRunSearchResponse = {
  results: EngineRankedJobResult[];
  meta: EngineSearchRunMeta;
};

type EngineRankedJobResult = {
  job: EngineJob;
  fit: EngineFitExplanation;
};

type EngineFitExplanation = {
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

type EngineSearchRunMeta = {
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

export type {
  EngineAnalyzeProfile,
  EngineApiError,
  EngineApplication,
  EngineApplicationDetail,
  EngineBuildSearchProfileResponse,
  EngineCompanyFeedbackRecord,
  EngineContact,
  EngineContactsResponse,
  EngineFeedbackOverviewResponse,
  EngineFitExplanation,
  EngineGlobalSearchApplication,
  EngineGlobalSearchResponse,
  EngineHealthResponse,
  EngineJob,
  EngineJobFeedSummary,
  EngineJobFeedbackRecord,
  EngineJobFeedbackState,
  EngineMarketCompaniesResponse,
  EngineMarketCompanyEntry,
  EngineMarketOverview,
  EngineMarketRoleDemandEntry,
  EngineMarketSalaryTrend,
  EngineMatchResult,
  EngineNotification,
  EngineNotificationsResponse,
  EngineOffer,
  EngineProfile,
  EngineRankedJobResult,
  EngineRecentApplicationsResponse,
  EngineRecentJobsResponse,
  EngineResume,
  EngineRoleCandidate,
  EngineRoleCatalogResponse,
  EngineRunSearchResponse,
  EngineSearchProfile,
  EngineSearchRoleCandidate,
  EngineSearchRunMeta,
  EngineSourceCatalogResponse,
  EngineUnreadNotificationsCountResponse,
};
