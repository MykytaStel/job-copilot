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
  Task,
  TaskInput,
  FeedbackOverview,
} from '@job-copilot/shared';
import {
  RECENT_JOBS_LIMIT_MAX,
  json,
  mlRequest,
  readStoredProfileId,
  request,
  requestOptional,
  resolveProfileId,
  unsupported,
  unsupportedPromise,
  withProfileIdQuery,
  writeStoredProfileId,
} from './api/client';
import type {
  EngineAnalyzeProfile,
  EngineApplication,
  EngineApplicationDetail,
  EngineBuildSearchProfileResponse,
  EngineCompanyFeedbackRecord,
  EngineContact,
  EngineContactsResponse,
  EngineFeedbackOverviewResponse,
  EngineFitExplanation,
  EngineGlobalSearchResponse,
  EngineHealthResponse,
  EngineJob,
  EngineJobFeedbackRecord,
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
  EngineRecentApplicationsResponse,
  EngineRecentJobsResponse,
  EngineResume,
  EngineRoleCandidate,
  EngineRoleCatalogResponse,
  EngineRunSearchResponse,
  EngineSearchRoleCandidate,
  EngineSourceCatalogResponse,
  EngineUnreadNotificationsCountResponse,
} from './api/engine-types';
import {
  mapApplication,
  mapApplicationDetail,
  mapCompanyFeedbackRecord,
  mapContact,
  mapJob,
  mapJobFeedSummary,
  mapJobFeedbackRecord,
  mapMatchResult,
  mapOffer,
  mapProfile,
  mapResume,
  normalizeMissingString,
  uniquePreservingOrder,
} from './api/mappers';
import type {
  AnalyticsFeedbackSummary,
  AnalyticsSummary,
  BehaviorSummary,
  FunnelSummary,
  LlmContext,
  LlmContextAnalyzedProfile,
  LlmContextEvidenceEntry,
} from './api/analytics';
import {
  buildApplicationCoachPayload,
  buildCoverLetterDraftPayload,
  buildInterviewPrepPayload,
  buildJobFitExplanationPayload,
  buildProfileInsightsPayload,
  buildWeeklyGuidancePayload,
  mapApplicationCoachResponse,
  mapCoverLetterDraftResponse,
  mapInterviewPrepResponse,
  mapJobFitExplanationResponse,
  mapProfileInsightsResponse,
  mapWeeklyGuidanceResponse,
} from './api/enrichment';
import type {
  MlApplicationCoachResponse,
  MlCoverLetterDraftResponse,
  MlInterviewPrepResponse,
  MlJobFitExplanationResponse,
  MlProfileInsightsResponse,
  MlWeeklyGuidanceResponse,
} from './api/enrichment';

export {
  getAnalyticsSummary,
  getBehaviorSummary,
  getFunnelSummary,
  getLlmContext,
} from './api/analytics';
export type {
  AnalyticsFeedbackSummary,
  AnalyticsSummary,
  BehaviorSignalCount,
  BehaviorSummary,
  FunnelConversionRates,
  FunnelSourceCountEntry,
  FunnelSummary,
  JobsByLifecycle,
  JobsBySourceEntry,
  LlmContext,
  LlmContextAnalyzedProfile,
  LlmContextEvidenceEntry,
} from './api/analytics';

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

export type MarketTrend = 'up' | 'down' | 'stable';

export type MarketOverview = {
  newJobsThisWeek: number;
  activeCompaniesCount: number;
  activeJobsCount: number;
  remotePercentage: number;
};

export type MarketCompany = {
  companyName: string;
  activeJobs: number;
  thisWeek: number;
  prevWeek: number;
  velocity: number;
};

export type MarketSalaryTrend = {
  seniority: string;
  p25: number;
  median: number;
  p75: number;
  sampleCount: number;
};

export type MarketRoleDemand = {
  roleGroup: string;
  thisPeriod: number;
  prevPeriod: number;
  trend: MarketTrend;
};

export type AppNotificationType =
  | 'new_jobs_found'
  | 'job_reactivated'
  | 'application_due_soon';

export type AppNotification = {
  id: string;
  profileId: string;
  type: AppNotificationType;
  title: string;
  body?: string;
  payload?: Record<string, unknown>;
  readAt?: string;
  createdAt: string;
};

const DEFAULT_MARKET_SENIORITY_BUCKETS = ['junior', 'middle', 'senior', 'lead'] as const;

export type RankedJob = {
  jobId: string;
  title: string;
  companyName: string;
  score: number;
  matchedTerms: string[];
  positiveReasons: string[];
  negativeReasons: string[];
  missingSignals: string[];
  descriptionQuality: string;
};

export type FitAnalysis = {
  profileId: string;
  jobId: string;
  score: number;
  matchedTerms: string[];
  matchedRoles: string[];
  matchedSkills: string[];
  matchedKeywords: string[];
  missingTerms: string[];
  descriptionQuality: string;
  positiveReasons: string[];
  negativeReasons: string[];
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

export type UserEventType =
  | 'job_impression'
  | 'job_opened'
  | 'job_saved'
  | 'job_unsaved'
  | 'job_hidden'
  | 'job_unhidden'
  | 'job_bad_fit'
  | 'job_bad_fit_removed'
  | 'company_whitelisted'
  | 'company_blacklisted'
  | 'search_run'
  | 'fit_explanation_requested'
  | 'application_coach_requested'
  | 'cover_letter_draft_requested'
  | 'interview_prep_requested'
  | 'application_created';

type UserEventLogInput = {
  eventType: UserEventType;
  jobId?: string;
  companyName?: string;
  source?: string;
  roleFamily?: string;
  payloadJson?: Record<string, unknown>;
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
  missingSignals: string[];
  sourceMatch: boolean;
  workModeMatch?: boolean;
  regionMatch?: boolean;
  descriptionQuality: string;
  positiveReasons: string[];
  negativeReasons: string[];
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
    lowEvidenceJobs: number;
    weakDescriptionJobs: number;
    roleMismatchJobs: number;
    seniorityMismatchJobs: number;
    sourceMismatchJobs: number;
    topMissingSignals: string[];
  };
};

export type GlobalSearchApplicationResult = {
  id: string;
  jobId: string;
  status: ApplicationStatus;
  appliedAt?: string;
  dueDate?: string;
  updatedAt: string;
  jobTitle: string;
  companyName: string;
};

export type GlobalSearchResults = {
  jobs: JobPosting[];
  applications: GlobalSearchApplicationResult[];
};

export async function rerankJobs(profileId: string, jobIds: string[]): Promise<RankedJob[]> {
  const uniqueJobIds = Array.from(
    new Set(jobIds.map((jobId) => jobId.trim()).filter(Boolean)),
  );
  if (uniqueJobIds.length === 0) {
    return [];
  }

  const response = await request<{
    profile_id: string;
    results: EngineFitExplanation[];
  }>(`/api/v1/profiles/${profileId}/jobs/match`, json('POST', {
    job_ids: uniqueJobIds,
  }));

  return response.results
    .map((fit) => ({
      jobId: fit.job_id,
      title: '',
      companyName: '',
      score: fit.score,
      matchedTerms: uniquePreservingOrder([
        ...fit.matched_roles,
        ...fit.matched_skills,
        ...fit.matched_keywords,
      ]),
      positiveReasons: fit.positive_reasons,
      negativeReasons: fit.negative_reasons,
      missingSignals: fit.missing_signals,
      descriptionQuality: fit.description_quality,
    }))
    .sort(
      (left, right) =>
        right.score - left.score ||
        left.jobId.localeCompare(right.jobId),
    );
}

export async function analyzeFit(profileId: string, jobId: string): Promise<FitAnalysis> {
  const result = await request<EngineFitExplanation>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/match`,
  );
  const matchedTerms = uniquePreservingOrder([
    ...result.matched_roles,
    ...result.matched_skills,
    ...result.matched_keywords,
  ]);
  const evidence = result.positive_reasons.length > 0 ? result.positive_reasons : result.reasons;

  return {
    profileId,
    jobId: result.job_id,
    score: result.score,
    matchedTerms,
    matchedRoles: result.matched_roles,
    matchedSkills: result.matched_skills,
    matchedKeywords: result.matched_keywords,
    missingTerms: result.missing_signals,
    descriptionQuality: result.description_quality,
    positiveReasons: result.positive_reasons,
    negativeReasons: result.negative_reasons,
    evidence,
  };
}

export async function logUserEvent(
  profileId: string,
  input: UserEventLogInput,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/events`,
    json('POST', {
      event_type: input.eventType,
      job_id: input.jobId,
      company_name: input.companyName,
      source: input.source,
      role_family: input.roleFamily,
      payload_json: input.payloadJson,
    }),
  );
}

function mapNotification(notification: EngineNotification): AppNotification {
  return {
    id: notification.id,
    profileId: notification.profile_id,
    type: notification.type,
    title: notification.title,
    body: notification.body ?? undefined,
    payload: notification.payload ?? undefined,
    readAt: notification.read_at ?? undefined,
    createdAt: notification.created_at,
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
  if (params?.limit) {
    qs.set('limit', String(Math.min(params.limit, RECENT_JOBS_LIMIT_MAX)));
  }
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
      seniority: normalizeMissingString(response.analyzed_profile.seniority) ?? '',
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
      source: result.job.primary_variant?.source ?? '',
      fit: {
        jobId: result.fit.job_id,
        score: result.fit.score,
        matchedRoles: result.fit.matched_roles,
        matchedSkills: result.fit.matched_skills,
        matchedKeywords: result.fit.matched_keywords,
        missingSignals: result.fit.missing_signals,
        sourceMatch: result.fit.source_match,
        workModeMatch: result.fit.work_mode_match ?? undefined,
        regionMatch: result.fit.region_match ?? undefined,
        descriptionQuality: result.fit.description_quality,
        positiveReasons: result.fit.positive_reasons,
        negativeReasons: result.fit.negative_reasons,
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
      lowEvidenceJobs: response.meta.low_evidence_jobs,
      weakDescriptionJobs: response.meta.weak_description_jobs,
      roleMismatchJobs: response.meta.role_mismatch_jobs,
      seniorityMismatchJobs: response.meta.seniority_mismatch_jobs,
      sourceMismatchJobs: response.meta.source_mismatch_jobs,
      topMissingSignals: response.meta.top_missing_signals,
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

export async function getNotifications(
  profileId?: string,
  limit: number = 20,
): Promise<AppNotification[]> {
  const resolvedProfileId = resolveProfileId(profileId);
  if (!resolvedProfileId) {
    return [];
  }

  const response = await request<EngineNotificationsResponse>(
    `/api/v1/notifications?profile_id=${encodeURIComponent(
      resolvedProfileId,
    )}&limit=${encodeURIComponent(String(limit))}`,
  );

  return response.notifications.map(mapNotification);
}

export async function markNotificationRead(id: string): Promise<AppNotification> {
  const notification = await request<EngineNotification>(
    `/api/v1/notifications/${encodeURIComponent(id)}/read`,
    json('POST', {}),
  );

  return mapNotification(notification);
}

export async function getUnreadCount(profileId?: string): Promise<number> {
  const resolvedProfileId = resolveProfileId(profileId);
  if (!resolvedProfileId) {
    return 0;
  }

  const response = await request<EngineUnreadNotificationsCountResponse>(
    `/api/v1/notifications/unread-count?profile_id=${encodeURIComponent(
      resolvedProfileId,
    )}`,
  );

  return response.unread_count;
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
    ...mapProfile(profile),
    summary: analyzed.summary,
    skills: analyzed.skills,
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

  const profileId = readStoredProfileId();
  if (profileId) {
    void logUserEvent(profileId, {
      eventType: 'application_created',
      jobId: payload.jobId,
      payloadJson: {
        application_id: application.id,
        status: payload.status,
        applied_at: payload.appliedAt ?? null,
      },
    }).catch(() => null);
  }

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

export async function getMarketOverview(): Promise<MarketOverview> {
  const response = await request<EngineMarketOverview>('/api/v1/market/overview');

  return {
    newJobsThisWeek: response.new_jobs_this_week,
    activeCompaniesCount: response.active_companies_count,
    activeJobsCount: response.active_jobs_count,
    remotePercentage: response.remote_percentage,
  };
}

export async function getMarketCompanies(limit = 10): Promise<MarketCompany[]> {
  const response = await request<EngineMarketCompaniesResponse>(
    `/api/v1/market/companies?limit=${encodeURIComponent(String(limit))}`,
  );

  return response.companies.map((company) => ({
    companyName: company.company_name,
    activeJobs: company.active_jobs,
    thisWeek: company.this_week,
    prevWeek: company.prev_week,
    velocity: company.velocity,
  }));
}

export async function getMarketSalaries(
  seniorityBuckets: readonly string[] = DEFAULT_MARKET_SENIORITY_BUCKETS,
): Promise<MarketSalaryTrend[]> {
  const buckets = Array.from(
    new Set(
      seniorityBuckets
        .map((bucket) => bucket.trim().toLowerCase())
        .filter(Boolean),
    ),
  );

  const response = await request<EngineMarketSalaryTrend[]>(
    '/api/v1/market/salary-trends',
  );

  const trendsBySeniority = new Map(
    response.map((trend) => [
      trend.seniority.toLowerCase(),
      {
        seniority: trend.seniority,
        p25: trend.p25,
        median: trend.median,
        p75: trend.p75,
        sampleCount: trend.sample_count,
      } satisfies MarketSalaryTrend,
    ]),
  );

  return buckets
    .map((bucket) => trendsBySeniority.get(bucket))
    .filter((result): result is MarketSalaryTrend => result !== undefined);
}

export async function getMarketRoles(period = 30): Promise<MarketRoleDemand[]> {
  const response = await request<EngineMarketRoleDemandEntry[]>(
    `/api/v1/market/roles?period=${encodeURIComponent(String(period))}`,
  );

  return response.map((entry) => ({
    roleGroup: entry.role_group,
    thisPeriod: entry.this_period,
    prevPeriod: entry.prev_period,
    trend: entry.trend,
  }));
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
export async function globalSearch(query: string): Promise<GlobalSearchResults> {
  const result = await request<EngineGlobalSearchResponse>(
    `/api/v1/search?q=${encodeURIComponent(query)}&limit=10`,
  );

  return {
    jobs: result.jobs.map(mapJob),
    applications: result.applications.map((application) => ({
      id: application.id,
      jobId: application.job_id,
      status: application.status,
      appliedAt: application.applied_at ?? undefined,
      dueDate: application.due_date ?? undefined,
      updatedAt: application.updated_at,
      jobTitle: application.job_title,
      companyName: application.company_name,
    })),
  };
}
export async function getContacts(): Promise<Contact[]> {
  const response = await request<EngineContactsResponse>('/api/v1/contacts');
  return response.contacts.map(mapContact);
}

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

export type WeeklyGuidance = {
  weeklySummary: string;
  whatIsWorking: string[];
  whatIsNotWorking: string[];
  recommendedSearchAdjustments: string[];
  recommendedSourceMoves: string[];
  recommendedRoleFocus: string[];
  funnelBottlenecks: string[];
  nextWeekPlan: string[];
};

export type WeeklyGuidanceRequest = {
  profileId: string;
  analyticsSummary: AnalyticsSummary;
  behaviorSummary: BehaviorSummary;
  funnelSummary: FunnelSummary;
  llmContext: LlmContext;
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

export async function getProfileInsights(
  context: LlmContext,
): Promise<ProfileInsights> {
  const response = await mlRequest<MlProfileInsightsResponse>(
    '/v1/enrichment/profile-insights',
    {
    method: 'POST',
    body: JSON.stringify(buildProfileInsightsPayload(context)),
  });

  return mapProfileInsightsResponse(response);
}

export async function getWeeklyGuidance(
  payload: WeeklyGuidanceRequest,
): Promise<WeeklyGuidance> {
  const response = await mlRequest<MlWeeklyGuidanceResponse>(
    '/v1/enrichment/weekly-guidance',
    {
    method: 'POST',
    body: JSON.stringify(buildWeeklyGuidancePayload(payload)),
  });

  return mapWeeklyGuidanceResponse(response);
}

export async function getJobFitExplanation(
  payload: JobFitExplanationRequest,
): Promise<JobFitExplanation> {
  void logUserEvent(payload.profileId, {
    eventType: 'fit_explanation_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      primary_role: payload.searchProfile?.primaryRole ?? null,
      has_feedback_state: Boolean(payload.feedbackState),
    },
  }).catch(() => null);

  const response = await mlRequest<MlJobFitExplanationResponse>(
    '/v1/enrichment/job-fit-explanation',
    {
    method: 'POST',
    body: JSON.stringify(buildJobFitExplanationPayload(payload)),
  });

  return mapJobFitExplanationResponse(response);
}

export async function getApplicationCoach(
  payload: ApplicationCoachRequest,
): Promise<ApplicationCoach> {
  void logUserEvent(payload.profileId, {
    eventType: 'application_coach_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      primary_role: payload.searchProfile?.primaryRole ?? null,
    },
  }).catch(() => null);

  const response = await mlRequest<MlApplicationCoachResponse>(
    '/v1/enrichment/application-coach',
    {
    method: 'POST',
    body: JSON.stringify(buildApplicationCoachPayload(payload)),
  });

  return mapApplicationCoachResponse(response);
}

export async function getCoverLetterDraft(
  payload: CoverLetterDraftRequest,
): Promise<CoverLetterDraft> {
  void logUserEvent(payload.profileId, {
    eventType: 'cover_letter_draft_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      has_application_coach: Boolean(payload.applicationCoach),
      has_raw_profile_text: Boolean(payload.rawProfileText),
    },
  }).catch(() => null);

  const response = await mlRequest<MlCoverLetterDraftResponse>(
    '/v1/enrichment/cover-letter-draft',
    {
    method: 'POST',
    body: JSON.stringify(buildCoverLetterDraftPayload(payload)),
  });

  return mapCoverLetterDraftResponse(response);
}

export async function getInterviewPrep(
  payload: InterviewPrepRequest,
): Promise<InterviewPrep> {
  void logUserEvent(payload.profileId, {
    eventType: 'interview_prep_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      has_application_coach: Boolean(payload.applicationCoach),
      has_cover_letter_draft: Boolean(payload.coverLetterDraft),
      has_raw_profile_text: Boolean(payload.rawProfileText),
    },
  }).catch(() => null);

  const response = await mlRequest<MlInterviewPrepResponse>(
    '/v1/enrichment/interview-prep',
    {
    method: 'POST',
    body: JSON.stringify(buildInterviewPrepPayload(payload)),
  });

  return mapInterviewPrepResponse(response);
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
