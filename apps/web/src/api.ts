import type {
  Activity,
  ActivityInput,
  Application,
  ApplicationContact,
  ApplicationDetail,
  ApplicationInput,
  ApplicationNote,
  ApplicationStatus,
  BackupMeta,
  CandidateProfile,
  CandidateProfileInput,
  Contact,
  CompanyFeedbackRecord,
  CompanyFeedbackStatus,
  ContactInput,
  CoverLetter,
  CoverLetterInput,
  DashboardStats,
  HealthResponse,
  ImportBatchResponse,
  InterviewQA,
  InterviewQAInput,
  JobFeedbackRecord,
  JobFeedbackState,
  JobFeedSummary,
  JobAlert,
  JobAlertInput,
  JobPosting,
  JobPostingInput,
  MatchResult,
  Offer,
  OfferInput,
  ResumeUploadInput,
  ResumeVersion,
  SearchResults,
  Task,
  TaskInput,
  FeedbackOverview,
} from '@job-copilot/shared';

export interface SkillStat {
  skill: string;
  count: number;
  pct: number;
  inResume: boolean;
}

export interface MarketInsights {
  totalJobs: number;
  coverageScore: number;
  topSkills: SkillStat[];
  hotGaps: string[];
  salaryMentions: string[];
}

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
  status: Offer['status'];
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
  target_regions: SearchTargetRegion[];
  work_modes: SearchWorkMode[];
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
  source_match: boolean;
  work_mode_match?: boolean | null;
  region_match?: boolean | null;
  reasons: string[];
};

type EngineSearchRunMeta = {
  total_candidates: number;
  filtered_out_by_source: number;
  filtered_out_hidden: number;
  filtered_out_company_blacklist: number;
  scored_jobs: number;
  returned_jobs: number;
};

const API_URL =
  import.meta.env.VITE_ENGINE_API_URL?.trim() || 'http://localhost:8080';
const ML_URL =
  import.meta.env.VITE_ML_URL?.trim() || 'http://localhost:8000';
const PROFILE_ID_KEY = 'engine_api_profile_id';

export type RankedJob = {
  jobId: string;
  title: string;
  companyName: string;
  score: number;
  matchedTerms: string[];
  evidence: string[];
};

export type FitAnalysis = {
  profileId: string;
  jobId: string;
  score: number;
  matchedTerms: string[];
  missingTerms: string[];
  evidence: string[];
};

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

export type SearchTargetRegion =
  | 'ua'
  | 'eu'
  | 'eu_remote'
  | 'poland'
  | 'germany'
  | 'uk'
  | 'us';

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

export type SearchRunRequest = {
  searchProfile: SearchProfileBuildResult['searchProfile'];
  profileId?: string;
  limit?: number;
};

export type FitExplanation = {
  jobId: string;
  score: number;
  matchedRoles: string[];
  matchedSkills: string[];
  matchedKeywords: string[];
  sourceMatch: boolean;
  workModeMatch?: boolean;
  regionMatch?: boolean;
  reasons: string[];
};

export type RankedJobResult = {
  job: JobPosting;
  source: string;
  fit: FitExplanation;
};

export type SearchRunResult = {
  results: RankedJobResult[];
  meta: {
    totalCandidates: number;
    filteredOutBySource: number;
    filteredOutHidden: number;
    filteredOutCompanyBlacklist: number;
    scoredJobs: number;
    returnedJobs: number;
  };
};

async function mlRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${ML_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { detail?: string };
    throw new Error(body.detail ?? `ML HTTP ${res.status}`);
  }
  return res.json();
}

export async function rerankJobs(profileId: string, jobIds: string[]): Promise<RankedJob[]> {
  const result = await mlRequest<{
    profile_id: string;
    jobs: Array<{
      job_id: string;
      title: string;
      company_name: string;
      score: number;
      matched_terms: string[];
      evidence: string[];
    }>;
  }>('/api/v1/rerank', {
    method: 'POST',
    body: JSON.stringify({ profile_id: profileId, job_ids: jobIds }),
  });
  return result.jobs.map((j) => ({
    jobId: j.job_id,
    title: j.title,
    companyName: j.company_name,
    score: j.score,
    matchedTerms: j.matched_terms,
    evidence: j.evidence,
  }));
}

export async function analyzeFit(profileId: string, jobId: string): Promise<FitAnalysis> {
  const result = await mlRequest<{
    profile_id: string;
    job_id: string;
    score: number;
    matched_terms: string[];
    missing_terms: string[];
    evidence: string[];
  }>('/api/v1/fit/analyze', {
    method: 'POST',
    body: JSON.stringify({ profile_id: profileId, job_id: jobId }),
  });
  return {
    profileId: result.profile_id,
    jobId: result.job_id,
    score: result.score,
    matchedTerms: result.matched_terms,
    missingTerms: result.missing_terms,
    evidence: result.evidence,
  };
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, init);
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

function json(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  };
}

function unsupported(feature: string): never {
  throw new Error(`${feature} is not supported by engine-api yet`);
}

function unsupportedPromise<T>(feature: string): Promise<T> {
  return Promise.reject(
    new Error(`${feature} is not supported by engine-api yet`),
  );
}

function readStoredProfileId() {
  return window.localStorage.getItem(PROFILE_ID_KEY);
}

function writeStoredProfileId(profileId: string) {
  window.localStorage.setItem(PROFILE_ID_KEY, profileId);
}

function withProfileIdQuery(path: string) {
  const profileId = readStoredProfileId();
  if (!profileId) return path;

  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}profile_id=${encodeURIComponent(profileId)}`;
}

function mapJob(job: EngineJob): JobPosting {
  return {
    id: job.id,
    source: 'manual',
    url: job.presentation.outbound_url ?? job.primary_variant?.source_url ?? undefined,
    title: job.presentation.title || job.title,
    company: job.presentation.company || job.company_name,
    description: job.description_text,
    notes: '',
    createdAt: job.posted_at ?? job.last_seen_at,
    postedAt: job.posted_at ?? undefined,
    firstSeenAt: job.first_seen_at,
    lastSeenAt: job.last_seen_at,
    isActive: job.is_active,
    inactivatedAt: job.inactivated_at ?? undefined,
    reactivatedAt: job.reactivated_at ?? undefined,
    lifecycleStage: job.lifecycle_stage,
    salaryMin: job.salary_min ?? undefined,
    salaryMax: job.salary_max ?? undefined,
    salaryCurrency: job.salary_currency ?? undefined,
    seniority: job.seniority ?? undefined,
    remoteType: job.remote_type ?? undefined,
    primaryVariant: job.primary_variant
      ? {
          source: job.primary_variant.source,
          sourceJobId: job.primary_variant.source_job_id,
          sourceUrl: job.primary_variant.source_url,
          fetchedAt: job.primary_variant.fetched_at,
          lastSeenAt: job.primary_variant.last_seen_at,
          isActive: job.primary_variant.is_active,
          inactivatedAt: job.primary_variant.inactivated_at ?? undefined,
        }
      : undefined,
    presentation: {
      title: job.presentation.title,
      company: job.presentation.company,
      summary: job.presentation.summary ?? undefined,
      locationLabel: job.presentation.location_label ?? undefined,
      workModeLabel: job.presentation.work_mode_label ?? undefined,
      sourceLabel: job.presentation.source_label ?? undefined,
      outboundUrl: job.presentation.outbound_url ?? undefined,
      salaryLabel: job.presentation.salary_label ?? undefined,
      freshnessLabel: job.presentation.freshness_label ?? undefined,
      badges: job.presentation.badges,
    },
    feedback: mapJobFeedbackState(job.feedback),
  };
}

function mapJobFeedbackState(feedback: EngineJobFeedbackState): JobFeedbackState {
  return {
    saved: feedback.saved,
    hidden: feedback.hidden,
    badFit: feedback.bad_fit,
    companyStatus: feedback.company_status ?? undefined,
  };
}

function mapJobFeedbackRecord(record: EngineJobFeedbackRecord): JobFeedbackRecord {
  return {
    jobId: record.job_id,
    saved: record.saved,
    hidden: record.hidden,
    badFit: record.bad_fit,
    updatedAt: record.updated_at,
  };
}

function mapCompanyFeedbackRecord(
  record: EngineCompanyFeedbackRecord,
): CompanyFeedbackRecord {
  return {
    companyName: record.company_name,
    normalizedCompanyName: record.normalized_company_name,
    status: record.status,
    updatedAt: record.updated_at,
  };
}

function mapJobFeedSummary(summary: EngineJobFeedSummary): JobFeedSummary {
  return {
    totalJobs: summary.total_jobs,
    activeJobs: summary.active_jobs,
    inactiveJobs: summary.inactive_jobs,
    reactivatedJobs: summary.reactivated_jobs,
  };
}

function mapApplication(application: EngineApplication): Application {
  return {
    id: application.id,
    jobId: application.job_id,
    resumeId: application.resume_id ?? undefined,
    status: application.status,
    appliedAt: application.applied_at ?? undefined,
    dueDate: application.due_date ?? undefined,
    updatedAt: application.updated_at,
  };
}

function mapProfile(profile: EngineProfile): CandidateProfile {
  return {
    id: profile.id,
    name: profile.name,
    email: profile.email,
    location: profile.location ?? undefined,
    summary: profile.analysis?.summary,
    skills: profile.analysis?.skills ?? [],
    updatedAt: profile.updated_at,
    skillsUpdatedAt: profile.skills_updated_at ?? undefined,
  };
}

function mapContact(contact: EngineContact): Contact {
  return {
    id: contact.id,
    name: contact.name,
    email: contact.email ?? undefined,
    phone: contact.phone ?? undefined,
    linkedinUrl: contact.linkedin_url ?? undefined,
    company: contact.company ?? undefined,
    role: contact.role ?? undefined,
    createdAt: contact.created_at,
  };
}

function mapOffer(offer: EngineOffer): Offer {
  return {
    id: offer.id,
    applicationId: offer.application_id,
    status: offer.status,
    compensationMin: offer.compensation_min ?? undefined,
    compensationMax: offer.compensation_max ?? undefined,
    compensationCurrency: offer.compensation_currency ?? undefined,
    startsAt: offer.starts_at ?? undefined,
    notes: offer.notes ?? undefined,
    createdAt: offer.created_at,
    updatedAt: offer.updated_at,
  };
}

function mapResume(resume: EngineResume): ResumeVersion {
  return {
    id: resume.id,
    version: resume.version,
    filename: resume.filename,
    rawText: resume.raw_text,
    isActive: resume.is_active,
    uploadedAt: resume.uploaded_at,
  };
}

function mapMatchResult(result: EngineMatchResult): MatchResult {
  return {
    id: result.id,
    jobId: result.job_id,
    resumeId: result.resume_id,
    score: result.score,
    matchedSkills: result.matched_skills,
    missingSkills: result.missing_skills,
    notes: result.notes,
    createdAt: result.created_at,
  };
}

function mapApplicationDetail(detail: EngineApplicationDetail): ApplicationDetail {
  return {
    ...mapApplication(detail),
    job: mapJob(detail.job),
    resume: detail.resume ? mapResume(detail.resume) : undefined,
    offer: detail.offer ? mapOffer(detail.offer) : undefined,
    notes: detail.notes.map((note) => ({
      id: note.id,
      applicationId: note.application_id,
      content: note.content,
      createdAt: note.created_at,
    })),
    contacts: detail.contacts.map((contact) => ({
      id: contact.id,
      applicationId: contact.application_id,
      relationship: contact.relationship as ApplicationContact['relationship'],
      contact: mapContact(contact.contact),
    })),
    activities: detail.activities.map((activity) => ({
      id: activity.id,
      applicationId: activity.application_id,
      type: activity.activity_type as Activity['type'],
      description: activity.description,
      happenedAt: activity.happened_at,
      createdAt: activity.created_at,
    })),
    tasks: detail.tasks.map((task) => ({
      id: task.id,
      applicationId: task.application_id,
      title: task.title,
      remindAt: task.remind_at ?? undefined,
      done: task.done,
      createdAt: task.created_at,
    })),
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

  return request<EngineAnalyzeProfile>(
    `/api/v1/profiles/${profileId}/analyze`,
    json('POST', {}),
  );
}

// Supported engine-api endpoints
export async function getHealth(): Promise<HealthResponse> {
  const health = await request<EngineHealthResponse>('/health');

  return {
    status: 'ok',
    service: `engine-api:${health.database.status}`,
    timestamp: new Date().toISOString(),
  };
}

export async function getJobs(): Promise<JobPosting[]> {
  const response = await request<EngineRecentJobsResponse>(
    withProfileIdQuery('/api/v1/jobs/recent'),
  );
  return response.jobs.map(mapJob);
}

export async function getJobsFeed(params?: {
  lifecycle?: string;
  source?: string;
  limit?: number;
}): Promise<{
  jobs: JobPosting[];
  summary: JobFeedSummary;
}> {
  const qs = new URLSearchParams();
  if (params?.lifecycle) qs.set('lifecycle', params.lifecycle);
  if (params?.source) qs.set('source', params.source);
  if (params?.limit) qs.set('limit', String(params.limit));
  const profileId = readStoredProfileId();
  if (profileId) qs.set('profile_id', profileId);
  const query = qs.toString();
  const response = await request<EngineRecentJobsResponse>(
    `/api/v1/jobs/recent${query ? `?${query}` : ''}`,
  );
  return {
    jobs: response.jobs.map(mapJob),
    summary: mapJobFeedSummary(response.summary),
  };
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
      seniority: response.analyzed_profile.seniority,
      skills: response.analyzed_profile.skills,
      keywords: response.analyzed_profile.keywords,
      roleCandidates: response.analyzed_profile.role_candidates ?? [],
      suggestedSearchTerms: response.analyzed_profile.suggested_search_terms ?? [],
    },
    searchProfile: {
      primaryRole: response.search_profile.primary_role,
      primaryRoleConfidence:
        response.search_profile.primary_role_confidence ?? undefined,
      targetRoles: response.search_profile.target_roles,
      roleCandidates: response.search_profile.role_candidates ?? [],
      seniority: response.search_profile.seniority,
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

export async function runSearch(
  payload: SearchRunRequest,
): Promise<SearchRunResult> {
  const profileId = readStoredProfileId();
  const response = await request<EngineRunSearchResponse>(
    '/api/v1/search/run',
    json('POST', {
      profile_id: payload.profileId ?? profileId ?? undefined,
      search_profile: {
        primary_role: payload.searchProfile.primaryRole,
        primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
        target_roles: payload.searchProfile.targetRoles,
        role_candidates: payload.searchProfile.roleCandidates,
        seniority: payload.searchProfile.seniority,
        target_regions: payload.searchProfile.targetRegions,
        work_modes: payload.searchProfile.workModes,
        allowed_sources: payload.searchProfile.allowedSources,
        profile_skills: payload.searchProfile.profileSkills,
        profile_keywords: payload.searchProfile.profileKeywords,
        search_terms: payload.searchProfile.searchTerms,
        exclude_terms: payload.searchProfile.excludeTerms,
      },
      limit: payload.limit,
    }),
  );

  return {
    results: response.results.map((result) => ({
      job: mapJob(result.job),
      source: result.job.primary_variant?.source ?? 'unknown',
      fit: {
        jobId: result.fit.job_id,
        score: result.fit.score,
        matchedRoles: result.fit.matched_roles,
        matchedSkills: result.fit.matched_skills,
        matchedKeywords: result.fit.matched_keywords,
        sourceMatch: result.fit.source_match,
        workModeMatch: result.fit.work_mode_match ?? undefined,
        regionMatch: result.fit.region_match ?? undefined,
        reasons: result.fit.reasons,
      },
    })),
    meta: {
      totalCandidates: response.meta.total_candidates,
      filteredOutBySource: response.meta.filtered_out_by_source,
      filteredOutHidden: response.meta.filtered_out_hidden,
      filteredOutCompanyBlacklist:
        response.meta.filtered_out_company_blacklist,
      scoredJobs: response.meta.scored_jobs,
      returnedJobs: response.meta.returned_jobs,
    },
  };
}

export async function getJob(id: string): Promise<JobPosting> {
  const job = await request<EngineJob>(withProfileIdQuery(`/api/v1/jobs/${id}`));
  return mapJob(job);
}

export async function getFeedback(profileId: string): Promise<FeedbackOverview> {
  const response = await request<EngineFeedbackOverviewResponse>(
    `/api/v1/profiles/${profileId}/feedback`,
  );

  return {
    profileId: response.profile_id,
    jobs: response.jobs.map(mapJobFeedbackRecord),
    companies: response.companies.map(mapCompanyFeedbackRecord),
    summary: {
      savedJobsCount: response.summary.saved_jobs_count,
      hiddenJobsCount: response.summary.hidden_jobs_count,
      badFitJobsCount: response.summary.bad_fit_jobs_count,
      whitelistedCompaniesCount: response.summary.whitelisted_companies_count,
      blacklistedCompaniesCount: response.summary.blacklisted_companies_count,
    },
  };
}

export async function markJobSaved(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/saved`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function hideJobForProfile(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/hidden`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function markJobBadFit(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/bad-fit`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function unsaveJob(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/saved`,
    { method: 'DELETE' },
  );
}

export async function unhideJob(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/hidden`,
    { method: 'DELETE' },
  );
}

export async function unmarkJobBadFit(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/bad-fit`,
    { method: 'DELETE' },
  );
}

export async function addCompanyWhitelist(
  profileId: string,
  companyName: string,
): Promise<CompanyFeedbackRecord> {
  const record = await request<EngineCompanyFeedbackRecord>(
    `/api/v1/profiles/${profileId}/companies/whitelist`,
    json('PUT', { company_name: companyName }),
  );

  return mapCompanyFeedbackRecord(record);
}

export async function removeCompanyWhitelist(
  profileId: string,
  companyName: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/companies/whitelist`,
    json('DELETE', { company_name: companyName }),
  );
}

export async function addCompanyBlacklist(
  profileId: string,
  companyName: string,
): Promise<CompanyFeedbackRecord> {
  const record = await request<EngineCompanyFeedbackRecord>(
    `/api/v1/profiles/${profileId}/companies/blacklist`,
    json('PUT', { company_name: companyName }),
  );

  return mapCompanyFeedbackRecord(record);
}

export async function removeCompanyBlacklist(
  profileId: string,
  companyName: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/companies/blacklist`,
    json('DELETE', { company_name: companyName }),
  );
}

export async function getApplications(): Promise<Application[]> {
  const response = await request<EngineRecentApplicationsResponse>(
    '/api/v1/applications/recent',
  );
  return response.applications.map(mapApplication);
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
    location: payload.location,
    raw_text: payload.rawText,
  };

  const profile = profileId
    ? await request<EngineProfile>(
        `/api/v1/profiles/${profileId}`,
        json('PATCH', body),
      )
    : await request<EngineProfile>('/api/v1/profiles', json('POST', body));

  writeStoredProfileId(profile.id);

  // Upload resume in parallel with analysis so the engine-api fit-score endpoint
  // has an active resume to work with (resumes and profiles are separate records).
  const [analyzed] = await Promise.all([
    request<EngineAnalyzeProfile>(`/api/v1/profiles/${profile.id}/analyze`, json('POST', {})),
    request<EngineResume>('/api/v1/resume/upload', json('POST', {
      filename: 'profile.md',
      raw_text: payload.rawText,
    })).catch(() => null), // best-effort; don't block profile save if this fails
  ]);

  return {
    id: profile.id,
    name: profile.name,
    email: profile.email,
    location: profile.location ?? undefined,
    summary: analyzed.summary,
    skills: analyzed.skills,
    updatedAt: profile.updated_at,
  };
}

export async function getDashboardStats(): Promise<DashboardStats> {
  const applications = await getApplications();

  const byStatus: DashboardStats['byStatus'] = {
    saved: 0,
    applied: 0,
    interview: 0,
    offer: 0,
    rejected: 0,
  };

  for (const application of applications) {
    byStatus[application.status] += 1;
  }

  return {
    total: applications.length,
    byStatus,
    topMissingSkills: [],
    avgScore: null,
    tasksDueSoon: 0,
  };
}

// Unsupported legacy endpoints kept only to avoid breaking compile-time imports.
export const createJob = (_payload: JobPostingInput): Promise<JobPosting> =>
  unsupportedPromise('Job creation');
export const fetchJobUrl = (
  _url: string,
): Promise<{ title: string; company: string; description: string }> =>
  unsupportedPromise('Job fetch by URL');
export async function getResumes(): Promise<ResumeVersion[]> {
  const resumes = await request<EngineResume[]>('/api/v1/resumes');
  return resumes.map(mapResume);
}

export async function getActiveResume(): Promise<ResumeVersion> {
  const resume = await request<EngineResume>('/api/v1/resumes/active');
  return mapResume(resume);
}

export async function uploadResume(
  payload: ResumeUploadInput,
): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    '/api/v1/resume/upload',
    json('POST', {
      filename: payload.filename,
      raw_text: payload.rawText,
    }),
  );
  return mapResume(resume);
}

export const uploadResumeFile = (_file: File): Promise<ResumeVersion> =>
  unsupportedPromise('Resume upload');
export async function activateResume(id: string): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    `/api/v1/resumes/${id}/activate`,
    json('POST', {}),
  );
  return mapResume(resume);
}

export async function runMatch(jobId: string): Promise<MatchResult> {
  const result = await request<EngineMatchResult>(
    `/api/v1/jobs/${jobId}/match`,
    json('POST', {}),
  );
  return mapMatchResult(result);
}

export async function getMatch(jobId: string): Promise<MatchResult> {
  const result = await request<EngineMatchResult>(`/api/v1/jobs/${jobId}/match`);
  return mapMatchResult(result);
}

export async function getApplicationDetail(id: string): Promise<ApplicationDetail> {
  const detail = await request<EngineApplicationDetail>(`/api/v1/applications/${id}`);
  return mapApplicationDetail(detail);
}

export async function createApplication(
  payload: ApplicationInput,
): Promise<Application> {
  const application = await request<EngineApplication>(
    '/api/v1/applications',
    json('POST', {
      job_id: payload.jobId,
      status: payload.status,
      applied_at: payload.appliedAt,
    }),
  );
  return mapApplication(application);
}

export async function updateApplication(
  id: string,
  payload: {
    status?: ApplicationStatus;
    dueDate?: string | null;
  },
): Promise<Application> {
  const body: {
    status?: ApplicationStatus;
    due_date?: string | null;
  } = {};

  if (payload.status !== undefined) {
    body.status = payload.status;
  }

  if (payload.dueDate !== undefined) {
    body.due_date = payload.dueDate;
  }

  const application = await request<EngineApplication>(
    `/api/v1/applications/${id}`,
    json('PATCH', body),
  );
  return mapApplication(application);
}

export async function patchApplication(
  id: string,
  status: ApplicationStatus,
): Promise<Application> {
  return updateApplication(id, { status });
}

export async function setDueDate(
  id: string,
  dueDate: string | null,
): Promise<Application> {
  return updateApplication(id, { dueDate });
}
export async function addNote(
  applicationId: string,
  content: string,
): Promise<ApplicationNote> {
  const note = await request<{
    id: string;
    application_id: string;
    content: string;
    created_at: string;
  }>(`/api/v1/applications/${applicationId}/notes`, json('POST', { content }));

  return {
    id: note.id,
    applicationId: note.application_id,
    content: note.content,
    createdAt: note.created_at,
  };
}
export const updateJobNote = (_id: string, _note: string): Promise<JobPosting> =>
  unsupported('Job notes');
export const deleteJob = (_id: string): Promise<void> => unsupported('Job deletion');
export const deleteApplication = (_id: string): Promise<void> =>
  unsupported('Application deletion');
export const getMarketInsights = (): Promise<MarketInsights> =>
  unsupported('Market insights');
export const getAlerts = (): Promise<JobAlert[]> => unsupported('Alerts');
export const createAlert = (_payload: JobAlertInput): Promise<JobAlert> =>
  unsupported('Alerts');
export const toggleAlert = (_id: string, _active: boolean): Promise<JobAlert> =>
  unsupported('Alerts');
export const deleteAlert = (_id: string): Promise<void> => unsupported('Alerts');
export const getSuggestedSkills = (): Promise<string[]> => unsupported('Suggested skills');
export async function search(q: string): Promise<SearchResults> {
  const result = await request<{
    jobs: EngineJob[];
    contacts: Array<{ id: string; name: string; role?: string | null; email?: string | null }>;
    page: number;
    per_page: number;
    has_more: boolean;
  }>(`/api/v1/search?q=${encodeURIComponent(q)}`);

  return {
    jobs: result.jobs.map(mapJob),
    contacts: result.contacts.map((contact) => ({
      id: contact.id,
      name: contact.name,
      role: contact.role ?? undefined,
      email: contact.email ?? undefined,
      createdAt: '',
    })),
    page: result.page,
    perPage: result.per_page,
    hasMore: result.has_more,
  };
}
export async function getContacts(): Promise<Contact[]> {
  const response = await request<EngineContactsResponse>('/api/v1/contacts');
  return response.contacts.map(mapContact);
}

// ─── Analytics types ──────────────────────────────────────────────────────────

type EngineJobsBySourceEntry = {
  source: string;
  count: number;
};

type EngineJobsByLifecycle = {
  total: number;
  active: number;
  inactive: number;
  reactivated: number;
};

type EngineFeedbackSummarySection = {
  saved_jobs_count: number;
  hidden_jobs_count: number;
  bad_fit_jobs_count: number;
  whitelisted_companies_count: number;
  blacklisted_companies_count: number;
};

type EngineAnalyticsSummaryResponse = {
  profile_id: string;
  feedback: EngineFeedbackSummarySection;
  jobs_by_source: EngineJobsBySourceEntry[];
  jobs_by_lifecycle: EngineJobsByLifecycle;
  top_matched_roles: string[];
  top_matched_skills: string[];
  top_matched_keywords: string[];
};

type EngineLlmContextAnalyzedProfile = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

type EngineLlmContextEvidenceEntry = {
  type: string;
  label: string;
};

type EngineLlmContextResponse = {
  profile_id: string;
  analyzed_profile: EngineLlmContextAnalyzedProfile | null;
  profile_skills: string[];
  profile_keywords: string[];
  jobs_feed_summary: EngineJobsByLifecycle;
  feedback_summary: EngineFeedbackSummarySection;
  top_positive_evidence: EngineLlmContextEvidenceEntry[];
  top_negative_evidence: EngineLlmContextEvidenceEntry[];
};

type MlProfileInsightsResponse = {
  profile_summary: string;
  search_strategy_summary: string;
  strengths: string[];
  risks: string[];
  recommended_actions: string[];
  top_focus_areas: string[];
  search_term_suggestions: string[];
  application_strategy: string[];
};

type MlJobFitExplanationResponse = {
  fit_summary: string;
  why_it_matches: string[];
  risks: string[];
  missing_signals: string[];
  recommended_next_step: string;
  application_angle: string;
};

type MlApplicationCoachResponse = {
  application_summary: string;
  resume_focus_points: string[];
  suggested_bullets: string[];
  cover_letter_angles: string[];
  interview_focus: string[];
  gaps_to_address: string[];
  red_flags: string[];
};

type MlCoverLetterDraftResponse = {
  draft_summary: string;
  opening_paragraph: string;
  body_paragraphs: string[];
  closing_paragraph: string;
  key_claims_used: string[];
  evidence_gaps: string[];
  tone_notes: string[];
};

type MlInterviewPrepResponse = {
  prep_summary: string;
  likely_topics: string[];
  technical_focus: string[];
  behavioral_focus: string[];
  stories_to_prepare: string[];
  questions_to_ask: string[];
  risk_areas: string[];
  follow_up_plan: string[];
};

export type JobsBySourceEntry = {
  source: string;
  count: number;
};

export type JobsByLifecycle = {
  total: number;
  active: number;
  inactive: number;
  reactivated: number;
};

export type AnalyticsFeedbackSummary = {
  savedJobsCount: number;
  hiddenJobsCount: number;
  badFitJobsCount: number;
  whitelistedCompaniesCount: number;
  blacklistedCompaniesCount: number;
};

export type AnalyticsSummary = {
  profileId: string;
  feedback: AnalyticsFeedbackSummary;
  jobsBySource: JobsBySourceEntry[];
  jobsByLifecycle: JobsByLifecycle;
  topMatchedRoles: string[];
  topMatchedSkills: string[];
  topMatchedKeywords: string[];
};

export type LlmContextEvidenceEntry = {
  type: string;
  label: string;
};

export type LlmContextAnalyzedProfile = {
  summary: string;
  primaryRole: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

export type LlmContext = {
  profileId: string;
  analyzedProfile: LlmContextAnalyzedProfile | null;
  profileSkills: string[];
  profileKeywords: string[];
  jobsFeedSummary: JobsByLifecycle;
  feedbackSummary: AnalyticsFeedbackSummary;
  topPositiveEvidence: LlmContextEvidenceEntry[];
  topNegativeEvidence: LlmContextEvidenceEntry[];
};

export type ProfileInsights = {
  profileSummary: string;
  searchStrategySummary: string;
  strengths: string[];
  risks: string[];
  recommendedActions: string[];
  topFocusAreas: string[];
  searchTermSuggestions: string[];
  applicationStrategy: string[];
};

export type JobFitExplanation = {
  fitSummary: string;
  whyItMatches: string[];
  risks: string[];
  missingSignals: string[];
  recommendedNextStep: string;
  applicationAngle: string;
};

export type ApplicationCoach = {
  applicationSummary: string;
  resumeFocusPoints: string[];
  suggestedBullets: string[];
  coverLetterAngles: string[];
  interviewFocus: string[];
  gapsToAddress: string[];
  redFlags: string[];
};

export type CoverLetterDraft = {
  draftSummary: string;
  openingParagraph: string;
  bodyParagraphs: string[];
  closingParagraph: string;
  keyClaimsUsed: string[];
  evidenceGaps: string[];
  toneNotes: string[];
};

export type InterviewPrep = {
  prepSummary: string;
  likelyTopics: string[];
  technicalFocus: string[];
  behavioralFocus: string[];
  storiesToPrepare: string[];
  questionsToAsk: string[];
  riskAreas: string[];
  followUpPlan: string[];
};

export type JobFitExplanationRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  feedbackState?: {
    feedbackSummary: AnalyticsFeedbackSummary;
    topPositiveEvidence: LlmContextEvidenceEntry[];
    topNegativeEvidence: LlmContextEvidenceEntry[];
    currentJobFeedback?: JobFeedbackState;
  } | null;
};

export type ApplicationCoachRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  feedbackState?: {
    feedbackSummary: AnalyticsFeedbackSummary;
    topPositiveEvidence: LlmContextEvidenceEntry[];
    topNegativeEvidence: LlmContextEvidenceEntry[];
    currentJobFeedback?: JobFeedbackState;
  } | null;
  rawProfileText?: string | null;
};

export type CoverLetterDraftRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  applicationCoach?: ApplicationCoach | null;
  feedbackState?: {
    feedbackSummary: AnalyticsFeedbackSummary;
    topPositiveEvidence: LlmContextEvidenceEntry[];
    topNegativeEvidence: LlmContextEvidenceEntry[];
    currentJobFeedback?: JobFeedbackState;
  } | null;
  rawProfileText?: string | null;
};

export type InterviewPrepRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  applicationCoach?: ApplicationCoach | null;
  coverLetterDraft?: CoverLetterDraft | null;
  feedbackState?: {
    feedbackSummary: AnalyticsFeedbackSummary;
    topPositiveEvidence: LlmContextEvidenceEntry[];
    topNegativeEvidence: LlmContextEvidenceEntry[];
    currentJobFeedback?: JobFeedbackState;
  } | null;
  rawProfileText?: string | null;
};

function mapFeedbackSummarySection(s: EngineFeedbackSummarySection): AnalyticsFeedbackSummary {
  return {
    savedJobsCount: s.saved_jobs_count,
    hiddenJobsCount: s.hidden_jobs_count,
    badFitJobsCount: s.bad_fit_jobs_count,
    whitelistedCompaniesCount: s.whitelisted_companies_count,
    blacklistedCompaniesCount: s.blacklisted_companies_count,
  };
}

function mapJobsByLifecycle(l: EngineJobsByLifecycle): JobsByLifecycle {
  return { total: l.total, active: l.active, inactive: l.inactive, reactivated: l.reactivated };
}

export async function getAnalyticsSummary(profileId: string): Promise<AnalyticsSummary> {
  const response = await request<EngineAnalyticsSummaryResponse>(
    `/api/v1/profiles/${profileId}/analytics/summary`,
  );

  return {
    profileId: response.profile_id,
    feedback: mapFeedbackSummarySection(response.feedback),
    jobsBySource: response.jobs_by_source,
    jobsByLifecycle: mapJobsByLifecycle(response.jobs_by_lifecycle),
    topMatchedRoles: response.top_matched_roles,
    topMatchedSkills: response.top_matched_skills,
    topMatchedKeywords: response.top_matched_keywords,
  };
}

export async function getLlmContext(profileId: string): Promise<LlmContext> {
  const response = await request<EngineLlmContextResponse>(
    `/api/v1/profiles/${profileId}/analytics/llm-context`,
  );

  return {
    profileId: response.profile_id,
    analyzedProfile: response.analyzed_profile
      ? {
          summary: response.analyzed_profile.summary,
          primaryRole: response.analyzed_profile.primary_role,
          seniority: response.analyzed_profile.seniority,
          skills: response.analyzed_profile.skills,
          keywords: response.analyzed_profile.keywords,
        }
      : null,
    profileSkills: response.profile_skills,
    profileKeywords: response.profile_keywords,
    jobsFeedSummary: mapJobsByLifecycle(response.jobs_feed_summary),
    feedbackSummary: mapFeedbackSummarySection(response.feedback_summary),
    topPositiveEvidence: response.top_positive_evidence,
    topNegativeEvidence: response.top_negative_evidence,
  };
}

export async function getProfileInsights(context: LlmContext): Promise<ProfileInsights> {
  const response = await mlRequest<MlProfileInsightsResponse>('/v1/enrichment/profile-insights', {
    method: 'POST',
    body: JSON.stringify({
      profile_id: context.profileId,
      analyzed_profile: context.analyzedProfile
        ? {
            summary: context.analyzedProfile.summary,
            primary_role: context.analyzedProfile.primaryRole,
            seniority: context.analyzedProfile.seniority,
            skills: context.analyzedProfile.skills,
            keywords: context.analyzedProfile.keywords,
          }
        : null,
      profile_skills: context.profileSkills,
      profile_keywords: context.profileKeywords,
      jobs_feed_summary: {
        total: context.jobsFeedSummary.total,
        active: context.jobsFeedSummary.active,
        inactive: context.jobsFeedSummary.inactive,
        reactivated: context.jobsFeedSummary.reactivated,
      },
      feedback_summary: {
        saved_jobs_count: context.feedbackSummary.savedJobsCount,
        hidden_jobs_count: context.feedbackSummary.hiddenJobsCount,
        bad_fit_jobs_count: context.feedbackSummary.badFitJobsCount,
        whitelisted_companies_count: context.feedbackSummary.whitelistedCompaniesCount,
        blacklisted_companies_count: context.feedbackSummary.blacklistedCompaniesCount,
      },
      top_positive_evidence: context.topPositiveEvidence.map((entry) => ({
        type: entry.type,
        label: entry.label,
      })),
      top_negative_evidence: context.topNegativeEvidence.map((entry) => ({
        type: entry.type,
        label: entry.label,
      })),
    }),
  });

  return {
    profileSummary: response.profile_summary,
    searchStrategySummary: response.search_strategy_summary,
    strengths: response.strengths,
    risks: response.risks,
    recommendedActions: response.recommended_actions,
    topFocusAreas: response.top_focus_areas,
    searchTermSuggestions: response.search_term_suggestions,
    applicationStrategy: response.application_strategy,
  };
}

export async function getJobFitExplanation(
  payload: JobFitExplanationRequest,
): Promise<JobFitExplanation> {
  const response = await mlRequest<MlJobFitExplanationResponse>('/v1/enrichment/job-fit-explanation', {
    method: 'POST',
    body: JSON.stringify({
      profile_id: payload.profileId,
      analyzed_profile: payload.analyzedProfile
        ? {
            summary: payload.analyzedProfile.summary,
            primary_role: payload.analyzedProfile.primaryRole,
            seniority: payload.analyzedProfile.seniority,
            skills: payload.analyzedProfile.skills,
            keywords: payload.analyzedProfile.keywords,
          }
        : null,
      search_profile: payload.searchProfile
        ? {
            primary_role: payload.searchProfile.primaryRole,
            primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
            target_roles: payload.searchProfile.targetRoles,
            role_candidates: payload.searchProfile.roleCandidates,
            seniority: payload.searchProfile.seniority,
            target_regions: payload.searchProfile.targetRegions,
            work_modes: payload.searchProfile.workModes,
            allowed_sources: payload.searchProfile.allowedSources,
            profile_skills: payload.searchProfile.profileSkills,
            profile_keywords: payload.searchProfile.profileKeywords,
            search_terms: payload.searchProfile.searchTerms,
            exclude_terms: payload.searchProfile.excludeTerms,
          }
        : null,
      ranked_job: {
        id: payload.rankedJob.id,
        title: payload.rankedJob.title,
        company_name: payload.rankedJob.company,
        description_text: payload.rankedJob.description,
        summary: payload.rankedJob.presentation?.summary,
        source: payload.rankedJob.primaryVariant?.source,
        source_job_id: payload.rankedJob.primaryVariant?.sourceJobId,
        source_url: payload.rankedJob.primaryVariant?.sourceUrl ?? payload.rankedJob.url,
        remote_type: payload.rankedJob.remoteType,
        seniority: payload.rankedJob.seniority,
        salary_label: payload.rankedJob.presentation?.salaryLabel,
        location_label: payload.rankedJob.presentation?.locationLabel,
        work_mode_label: payload.rankedJob.presentation?.workModeLabel,
        freshness_label: payload.rankedJob.presentation?.freshnessLabel,
        badges: payload.rankedJob.presentation?.badges ?? [],
      },
      deterministic_fit: {
        job_id: payload.deterministicFit.jobId,
        score: payload.deterministicFit.score,
        matched_roles: payload.deterministicFit.matchedRoles,
        matched_skills: payload.deterministicFit.matchedSkills,
        matched_keywords: payload.deterministicFit.matchedKeywords,
        source_match: payload.deterministicFit.sourceMatch,
        work_mode_match: payload.deterministicFit.workModeMatch,
        region_match: payload.deterministicFit.regionMatch,
        reasons: payload.deterministicFit.reasons,
      },
      feedback_state: payload.feedbackState
        ? {
            summary: {
              saved_jobs_count: payload.feedbackState.feedbackSummary.savedJobsCount,
              hidden_jobs_count: payload.feedbackState.feedbackSummary.hiddenJobsCount,
              bad_fit_jobs_count: payload.feedbackState.feedbackSummary.badFitJobsCount,
              whitelisted_companies_count:
                payload.feedbackState.feedbackSummary.whitelistedCompaniesCount,
              blacklisted_companies_count:
                payload.feedbackState.feedbackSummary.blacklistedCompaniesCount,
            },
            top_positive_evidence: payload.feedbackState.topPositiveEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            top_negative_evidence: payload.feedbackState.topNegativeEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            current_job_feedback: payload.feedbackState.currentJobFeedback
              ? {
                  saved: payload.feedbackState.currentJobFeedback.saved,
                  hidden: payload.feedbackState.currentJobFeedback.hidden,
                  bad_fit: payload.feedbackState.currentJobFeedback.badFit,
                  company_status: payload.feedbackState.currentJobFeedback.companyStatus,
                }
              : null,
          }
        : null,
    }),
  });

  return {
    fitSummary: response.fit_summary,
    whyItMatches: response.why_it_matches,
    risks: response.risks,
    missingSignals: response.missing_signals,
    recommendedNextStep: response.recommended_next_step,
    applicationAngle: response.application_angle,
  };
}

export async function getApplicationCoach(
  payload: ApplicationCoachRequest,
): Promise<ApplicationCoach> {
  const response = await mlRequest<MlApplicationCoachResponse>('/v1/enrichment/application-coach', {
    method: 'POST',
    body: JSON.stringify({
      profile_id: payload.profileId,
      analyzed_profile: payload.analyzedProfile
        ? {
            summary: payload.analyzedProfile.summary,
            primary_role: payload.analyzedProfile.primaryRole,
            seniority: payload.analyzedProfile.seniority,
            skills: payload.analyzedProfile.skills,
            keywords: payload.analyzedProfile.keywords,
          }
        : null,
      search_profile: payload.searchProfile
        ? {
            primary_role: payload.searchProfile.primaryRole,
            primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
            target_roles: payload.searchProfile.targetRoles,
            role_candidates: payload.searchProfile.roleCandidates,
            seniority: payload.searchProfile.seniority,
            target_regions: payload.searchProfile.targetRegions,
            work_modes: payload.searchProfile.workModes,
            allowed_sources: payload.searchProfile.allowedSources,
            profile_skills: payload.searchProfile.profileSkills,
            profile_keywords: payload.searchProfile.profileKeywords,
            search_terms: payload.searchProfile.searchTerms,
            exclude_terms: payload.searchProfile.excludeTerms,
          }
        : null,
      ranked_job: {
        id: payload.rankedJob.id,
        title: payload.rankedJob.title,
        company_name: payload.rankedJob.company,
        description_text: payload.rankedJob.description,
        summary: payload.rankedJob.presentation?.summary,
        source: payload.rankedJob.primaryVariant?.source,
        source_job_id: payload.rankedJob.primaryVariant?.sourceJobId,
        source_url: payload.rankedJob.primaryVariant?.sourceUrl ?? payload.rankedJob.url,
        remote_type: payload.rankedJob.remoteType,
        seniority: payload.rankedJob.seniority,
        salary_label: payload.rankedJob.presentation?.salaryLabel,
        location_label: payload.rankedJob.presentation?.locationLabel,
        work_mode_label: payload.rankedJob.presentation?.workModeLabel,
        freshness_label: payload.rankedJob.presentation?.freshnessLabel,
        badges: payload.rankedJob.presentation?.badges ?? [],
      },
      deterministic_fit: {
        job_id: payload.deterministicFit.jobId,
        score: payload.deterministicFit.score,
        matched_roles: payload.deterministicFit.matchedRoles,
        matched_skills: payload.deterministicFit.matchedSkills,
        matched_keywords: payload.deterministicFit.matchedKeywords,
        source_match: payload.deterministicFit.sourceMatch,
        work_mode_match: payload.deterministicFit.workModeMatch,
        region_match: payload.deterministicFit.regionMatch,
        reasons: payload.deterministicFit.reasons,
      },
      job_fit_explanation: payload.jobFitExplanation
        ? {
            fit_summary: payload.jobFitExplanation.fitSummary,
            why_it_matches: payload.jobFitExplanation.whyItMatches,
            risks: payload.jobFitExplanation.risks,
            missing_signals: payload.jobFitExplanation.missingSignals,
            recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
            application_angle: payload.jobFitExplanation.applicationAngle,
          }
        : null,
      feedback_state: payload.feedbackState
        ? {
            summary: {
              saved_jobs_count: payload.feedbackState.feedbackSummary.savedJobsCount,
              hidden_jobs_count: payload.feedbackState.feedbackSummary.hiddenJobsCount,
              bad_fit_jobs_count: payload.feedbackState.feedbackSummary.badFitJobsCount,
              whitelisted_companies_count:
                payload.feedbackState.feedbackSummary.whitelistedCompaniesCount,
              blacklisted_companies_count:
                payload.feedbackState.feedbackSummary.blacklistedCompaniesCount,
            },
            top_positive_evidence: payload.feedbackState.topPositiveEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            top_negative_evidence: payload.feedbackState.topNegativeEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            current_job_feedback: payload.feedbackState.currentJobFeedback
              ? {
                  saved: payload.feedbackState.currentJobFeedback.saved,
                  hidden: payload.feedbackState.currentJobFeedback.hidden,
                  bad_fit: payload.feedbackState.currentJobFeedback.badFit,
                  company_status: payload.feedbackState.currentJobFeedback.companyStatus,
                }
              : null,
          }
        : null,
      raw_profile_text: payload.rawProfileText ?? null,
    }),
  });

  return {
    applicationSummary: response.application_summary,
    resumeFocusPoints: response.resume_focus_points,
    suggestedBullets: response.suggested_bullets,
    coverLetterAngles: response.cover_letter_angles,
    interviewFocus: response.interview_focus,
    gapsToAddress: response.gaps_to_address,
    redFlags: response.red_flags,
  };
}

export async function getCoverLetterDraft(
  payload: CoverLetterDraftRequest,
): Promise<CoverLetterDraft> {
  const response = await mlRequest<MlCoverLetterDraftResponse>('/v1/enrichment/cover-letter-draft', {
    method: 'POST',
    body: JSON.stringify({
      profile_id: payload.profileId,
      analyzed_profile: payload.analyzedProfile
        ? {
            summary: payload.analyzedProfile.summary,
            primary_role: payload.analyzedProfile.primaryRole,
            seniority: payload.analyzedProfile.seniority,
            skills: payload.analyzedProfile.skills,
            keywords: payload.analyzedProfile.keywords,
          }
        : null,
      search_profile: payload.searchProfile
        ? {
            primary_role: payload.searchProfile.primaryRole,
            primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
            target_roles: payload.searchProfile.targetRoles,
            role_candidates: payload.searchProfile.roleCandidates,
            seniority: payload.searchProfile.seniority,
            target_regions: payload.searchProfile.targetRegions,
            work_modes: payload.searchProfile.workModes,
            allowed_sources: payload.searchProfile.allowedSources,
            profile_skills: payload.searchProfile.profileSkills,
            profile_keywords: payload.searchProfile.profileKeywords,
            search_terms: payload.searchProfile.searchTerms,
            exclude_terms: payload.searchProfile.excludeTerms,
          }
        : null,
      ranked_job: {
        id: payload.rankedJob.id,
        title: payload.rankedJob.title,
        company_name: payload.rankedJob.company,
        description_text: payload.rankedJob.description,
        summary: payload.rankedJob.presentation?.summary,
        source: payload.rankedJob.primaryVariant?.source,
        source_job_id: payload.rankedJob.primaryVariant?.sourceJobId,
        source_url: payload.rankedJob.primaryVariant?.sourceUrl ?? payload.rankedJob.url,
        remote_type: payload.rankedJob.remoteType,
        seniority: payload.rankedJob.seniority,
        salary_label: payload.rankedJob.presentation?.salaryLabel,
        location_label: payload.rankedJob.presentation?.locationLabel,
        work_mode_label: payload.rankedJob.presentation?.workModeLabel,
        freshness_label: payload.rankedJob.presentation?.freshnessLabel,
        badges: payload.rankedJob.presentation?.badges ?? [],
      },
      deterministic_fit: {
        job_id: payload.deterministicFit.jobId,
        score: payload.deterministicFit.score,
        matched_roles: payload.deterministicFit.matchedRoles,
        matched_skills: payload.deterministicFit.matchedSkills,
        matched_keywords: payload.deterministicFit.matchedKeywords,
        source_match: payload.deterministicFit.sourceMatch,
        work_mode_match: payload.deterministicFit.workModeMatch,
        region_match: payload.deterministicFit.regionMatch,
        reasons: payload.deterministicFit.reasons,
      },
      job_fit_explanation: payload.jobFitExplanation
        ? {
            fit_summary: payload.jobFitExplanation.fitSummary,
            why_it_matches: payload.jobFitExplanation.whyItMatches,
            risks: payload.jobFitExplanation.risks,
            missing_signals: payload.jobFitExplanation.missingSignals,
            recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
            application_angle: payload.jobFitExplanation.applicationAngle,
          }
        : null,
      application_coach: payload.applicationCoach
        ? {
            application_summary: payload.applicationCoach.applicationSummary,
            resume_focus_points: payload.applicationCoach.resumeFocusPoints,
            suggested_bullets: payload.applicationCoach.suggestedBullets,
            cover_letter_angles: payload.applicationCoach.coverLetterAngles,
            interview_focus: payload.applicationCoach.interviewFocus,
            gaps_to_address: payload.applicationCoach.gapsToAddress,
            red_flags: payload.applicationCoach.redFlags,
          }
        : null,
      feedback_state: payload.feedbackState
        ? {
            summary: {
              saved_jobs_count: payload.feedbackState.feedbackSummary.savedJobsCount,
              hidden_jobs_count: payload.feedbackState.feedbackSummary.hiddenJobsCount,
              bad_fit_jobs_count: payload.feedbackState.feedbackSummary.badFitJobsCount,
              whitelisted_companies_count:
                payload.feedbackState.feedbackSummary.whitelistedCompaniesCount,
              blacklisted_companies_count:
                payload.feedbackState.feedbackSummary.blacklistedCompaniesCount,
            },
            top_positive_evidence: payload.feedbackState.topPositiveEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            top_negative_evidence: payload.feedbackState.topNegativeEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            current_job_feedback: payload.feedbackState.currentJobFeedback
              ? {
                  saved: payload.feedbackState.currentJobFeedback.saved,
                  hidden: payload.feedbackState.currentJobFeedback.hidden,
                  bad_fit: payload.feedbackState.currentJobFeedback.badFit,
                  company_status: payload.feedbackState.currentJobFeedback.companyStatus,
                }
              : null,
          }
        : null,
      raw_profile_text: payload.rawProfileText ?? null,
    }),
  });

  return {
    draftSummary: response.draft_summary,
    openingParagraph: response.opening_paragraph,
    bodyParagraphs: response.body_paragraphs,
    closingParagraph: response.closing_paragraph,
    keyClaimsUsed: response.key_claims_used,
    evidenceGaps: response.evidence_gaps,
    toneNotes: response.tone_notes,
  };
}

export async function getInterviewPrep(
  payload: InterviewPrepRequest,
): Promise<InterviewPrep> {
  const response = await mlRequest<MlInterviewPrepResponse>('/v1/enrichment/interview-prep', {
    method: 'POST',
    body: JSON.stringify({
      profile_id: payload.profileId,
      analyzed_profile: payload.analyzedProfile
        ? {
            summary: payload.analyzedProfile.summary,
            primary_role: payload.analyzedProfile.primaryRole,
            seniority: payload.analyzedProfile.seniority,
            skills: payload.analyzedProfile.skills,
            keywords: payload.analyzedProfile.keywords,
          }
        : null,
      search_profile: payload.searchProfile
        ? {
            primary_role: payload.searchProfile.primaryRole,
            primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
            target_roles: payload.searchProfile.targetRoles,
            role_candidates: payload.searchProfile.roleCandidates,
            seniority: payload.searchProfile.seniority,
            target_regions: payload.searchProfile.targetRegions,
            work_modes: payload.searchProfile.workModes,
            allowed_sources: payload.searchProfile.allowedSources,
            profile_skills: payload.searchProfile.profileSkills,
            profile_keywords: payload.searchProfile.profileKeywords,
            search_terms: payload.searchProfile.searchTerms,
            exclude_terms: payload.searchProfile.excludeTerms,
          }
        : null,
      ranked_job: {
        id: payload.rankedJob.id,
        title: payload.rankedJob.title,
        company_name: payload.rankedJob.company,
        description_text: payload.rankedJob.description,
        summary: payload.rankedJob.presentation?.summary,
        source: payload.rankedJob.primaryVariant?.source,
        source_job_id: payload.rankedJob.primaryVariant?.sourceJobId,
        source_url: payload.rankedJob.primaryVariant?.sourceUrl ?? payload.rankedJob.url,
        remote_type: payload.rankedJob.remoteType,
        seniority: payload.rankedJob.seniority,
        salary_label: payload.rankedJob.presentation?.salaryLabel,
        location_label: payload.rankedJob.presentation?.locationLabel,
        work_mode_label: payload.rankedJob.presentation?.workModeLabel,
        freshness_label: payload.rankedJob.presentation?.freshnessLabel,
        badges: payload.rankedJob.presentation?.badges ?? [],
      },
      deterministic_fit: {
        job_id: payload.deterministicFit.jobId,
        score: payload.deterministicFit.score,
        matched_roles: payload.deterministicFit.matchedRoles,
        matched_skills: payload.deterministicFit.matchedSkills,
        matched_keywords: payload.deterministicFit.matchedKeywords,
        source_match: payload.deterministicFit.sourceMatch,
        work_mode_match: payload.deterministicFit.workModeMatch,
        region_match: payload.deterministicFit.regionMatch,
        reasons: payload.deterministicFit.reasons,
      },
      job_fit_explanation: payload.jobFitExplanation
        ? {
            fit_summary: payload.jobFitExplanation.fitSummary,
            why_it_matches: payload.jobFitExplanation.whyItMatches,
            risks: payload.jobFitExplanation.risks,
            missing_signals: payload.jobFitExplanation.missingSignals,
            recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
            application_angle: payload.jobFitExplanation.applicationAngle,
          }
        : null,
      application_coach: payload.applicationCoach
        ? {
            application_summary: payload.applicationCoach.applicationSummary,
            resume_focus_points: payload.applicationCoach.resumeFocusPoints,
            suggested_bullets: payload.applicationCoach.suggestedBullets,
            cover_letter_angles: payload.applicationCoach.coverLetterAngles,
            interview_focus: payload.applicationCoach.interviewFocus,
            gaps_to_address: payload.applicationCoach.gapsToAddress,
            red_flags: payload.applicationCoach.redFlags,
          }
        : null,
      cover_letter_draft: payload.coverLetterDraft
        ? {
            draft_summary: payload.coverLetterDraft.draftSummary,
            opening_paragraph: payload.coverLetterDraft.openingParagraph,
            body_paragraphs: payload.coverLetterDraft.bodyParagraphs,
            closing_paragraph: payload.coverLetterDraft.closingParagraph,
            key_claims_used: payload.coverLetterDraft.keyClaimsUsed,
            evidence_gaps: payload.coverLetterDraft.evidenceGaps,
            tone_notes: payload.coverLetterDraft.toneNotes,
          }
        : null,
      feedback_state: payload.feedbackState
        ? {
            summary: {
              saved_jobs_count: payload.feedbackState.feedbackSummary.savedJobsCount,
              hidden_jobs_count: payload.feedbackState.feedbackSummary.hiddenJobsCount,
              bad_fit_jobs_count: payload.feedbackState.feedbackSummary.badFitJobsCount,
              whitelisted_companies_count:
                payload.feedbackState.feedbackSummary.whitelistedCompaniesCount,
              blacklisted_companies_count:
                payload.feedbackState.feedbackSummary.blacklistedCompaniesCount,
            },
            top_positive_evidence: payload.feedbackState.topPositiveEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            top_negative_evidence: payload.feedbackState.topNegativeEvidence.map((entry) => ({
              type: entry.type,
              label: entry.label,
            })),
            current_job_feedback: payload.feedbackState.currentJobFeedback
              ? {
                  saved: payload.feedbackState.currentJobFeedback.saved,
                  hidden: payload.feedbackState.currentJobFeedback.hidden,
                  bad_fit: payload.feedbackState.currentJobFeedback.badFit,
                  company_status: payload.feedbackState.currentJobFeedback.companyStatus,
                }
              : null,
          }
        : null,
      raw_profile_text: payload.rawProfileText ?? null,
    }),
  });

  return {
    prepSummary: response.prep_summary,
    likelyTopics: response.likely_topics,
    technicalFocus: response.technical_focus,
    behavioralFocus: response.behavioral_focus,
    storiesToPrepare: response.stories_to_prepare,
    questionsToAsk: response.questions_to_ask,
    riskAreas: response.risk_areas,
    followUpPlan: response.follow_up_plan,
  };
}

export async function createContact(payload: ContactInput): Promise<Contact> {
  const contact = await request<EngineContact>(
    '/api/v1/contacts',
    json('POST', {
      name: payload.name,
      email: payload.email,
      phone: payload.phone,
      linkedin_url: payload.linkedinUrl,
      company: payload.company,
      role: payload.role,
    }),
  );

  return mapContact(contact);
}
export const updateContact = (
  _id: string,
  _payload: Partial<ContactInput>,
): Promise<Contact> => unsupported('Contacts');
export const deleteContact = (_id: string): Promise<void> => unsupported('Contacts');
export async function linkContact(
  applicationId: string,
  contactId: string,
  relationship: ApplicationContact['relationship'],
): Promise<ApplicationContact> {
  const contact = await request<{
    id: string;
    application_id: string;
    relationship: ApplicationContact['relationship'];
    contact: EngineContact;
  }>(
    `/api/v1/applications/${applicationId}/contacts`,
    json('POST', {
      contact_id: contactId,
      relationship,
    }),
  );

  return {
    id: contact.id,
    applicationId: contact.application_id,
    relationship: contact.relationship,
    contact: mapContact(contact.contact),
  };
}
export const unlinkContact = (
  _applicationId: string,
  _linkId: string,
): Promise<void> => unsupported('Application contacts');
export const getActivities = (_applicationId: string): Promise<Activity[]> =>
  unsupported('Activities');
export const createActivity = (
  _applicationId: string,
  _payload: ActivityInput,
): Promise<Activity> => unsupported('Activities');
export const deleteActivity = (_id: string): Promise<void> =>
  unsupported('Activities');
export const getTasks = (_applicationId: string): Promise<Task[]> => unsupported('Tasks');
export const getDueTasks = (): Promise<Task[]> => unsupported('Tasks');
export const createTask = (_applicationId: string, _payload: TaskInput): Promise<Task> =>
  unsupported('Tasks');
export const patchTask = (
  _id: string,
  _patch: { title?: string; remindAt?: string | null; done?: boolean },
): Promise<Task> => unsupported('Tasks');
export const deleteTask = (_id: string): Promise<void> => unsupported('Tasks');
export const getCoverLetters = (_jobId?: string): Promise<CoverLetter[]> =>
  unsupported('Cover letters');
export const createCoverLetter = (_payload: CoverLetterInput): Promise<CoverLetter> =>
  unsupported('Cover letters');
export const updateCoverLetter = (_id: string, _content: string): Promise<CoverLetter> =>
  unsupported('Cover letters');
export const deleteCoverLetter = (_id: string): Promise<void> =>
  unsupported('Cover letters');
export const getInterviewQA = (_jobId?: string): Promise<InterviewQA[]> =>
  unsupported('Interview Q&A');
export const createInterviewQA = (_payload: InterviewQAInput): Promise<InterviewQA> =>
  unsupported('Interview Q&A');
export const updateInterviewQA = (
  _id: string,
  _patch: { question?: string; answer?: string },
): Promise<InterviewQA> => unsupported('Interview Q&A');
export const deleteInterviewQA = (_id: string): Promise<void> =>
  unsupported('Interview Q&A');
export const getOffers = (): Promise<Offer[]> => unsupported('Offers');
export async function createOffer(payload: OfferInput): Promise<Offer> {
  const offer = await request<EngineOffer>(
    `/api/v1/applications/${payload.applicationId}/offer`,
    json('PUT', {
      status: payload.status,
      compensation_min: payload.compensationMin,
      compensation_max: payload.compensationMax,
      compensation_currency: payload.compensationCurrency,
      starts_at: payload.startsAt,
      notes: payload.notes,
    }),
  );

  return mapOffer(offer);
}
export const deleteOffer = (_id: string): Promise<void> => unsupported('Offers');
export const importBatch = (_urls: string[]): Promise<ImportBatchResponse> =>
  unsupported('Batch import');
export const downloadBackup = (): Promise<Record<string, unknown> & BackupMeta> =>
  unsupported('Backup');
export const restoreBackup = (
  _data: unknown,
): Promise<{ restored: boolean; exportedAt: string }> => unsupported('Backup');
