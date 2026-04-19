import type {
  CompanyFeedbackRecord,
  FeedbackOverview,
  HealthResponse,
  JobFeedbackRecord,
  JobFeedbackState,
  JobPosting,
} from '@job-copilot/shared';
import {
  json,
  mlRequest,
  readStoredProfileId,
  request,
} from './api/client';
import type {
  EngineCompanyFeedbackRecord,
  EngineFeedbackOverviewResponse,
  EngineHealthResponse,
  EngineJobFeedbackRecord,
  EngineNotification,
  EngineNotificationsResponse,
  EngineUnreadNotificationsCountResponse,
} from './api/engine-types';
import {
  mapCompanyFeedbackRecord,
  mapJobFeedbackRecord,
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
import { logUserEvent } from './api/events';
import type { SearchProfileBuildResult } from './api/profiles';
import type { FitExplanation } from './api/jobs';

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

export { logUserEvent } from './api/events';
export type { UserEventType } from './api/events';

export {
  getApplications,
  getApplicationDetail,
  createApplication,
  updateApplication,
  patchApplication,
  setDueDate,
  addNote,
  getDashboardStats,
  createOffer,
  deleteApplication,
  getActivities,
  createActivity,
  deleteActivity,
  getTasks,
  getDueTasks,
  createTask,
  patchTask,
  deleteTask,
  getCoverLetters,
  createCoverLetter,
  updateCoverLetter,
  deleteCoverLetter,
  getInterviewQA,
  createInterviewQA,
  updateInterviewQA,
  deleteInterviewQA,
  getOffers,
  deleteOffer,
} from './api/applications';
export type { GlobalSearchApplicationResult } from './api/applications';

export {
  getContacts,
  createContact,
  updateContact,
  deleteContact,
  linkContact,
  unlinkContact,
} from './api/contacts';

export {
  getMarketOverview,
  getMarketCompanies,
  getMarketSalaries,
  getMarketRoles,
  getMarketInsights,
} from './api/market';
export type {
  SkillStat,
  MarketInsights,
  MarketTrend,
  MarketOverview,
  MarketCompany,
  MarketSalaryTrend,
  MarketRoleDemand,
} from './api/market';

export {
  getStoredProfileRawText,
  analyzeStoredProfile,
  getProfile,
  saveProfile,
  getResumes,
  getActiveResume,
  uploadResume,
  activateResume,
  getSources,
  getRoles,
  buildSearchProfile,
} from './api/profiles';
export type {
  SourceCatalogItem,
  RoleCatalogItem,
  SearchTargetRegion,
  SearchWorkMode,
  SearchProfileBuildRequest,
  SearchProfileBuildResult,
} from './api/profiles';

export {
  getJobs,
  getJobsFeed,
  getJob,
  runSearch,
  rerankJobs,
  analyzeFit,
  runMatch,
  getMatch,
  globalSearch,
  createJob,
  fetchJobUrl,
  uploadResumeFile,
  updateJobNote,
  deleteJob,
  getAlerts,
  createAlert,
  toggleAlert,
  deleteAlert,
  getSuggestedSkills,
  importBatch,
  downloadBackup,
  restoreBackup,
} from './api/jobs';
export type {
  RankedJob,
  FitAnalysis,
  SearchRunRequest,
  FitExplanation,
  RankedJobResult,
  SearchRunResult,
  GlobalSearchResults,
} from './api/jobs';

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

export async function getHealth(): Promise<HealthResponse> {
  const health = await request<EngineHealthResponse>('/health');

  return {
    status: 'ok',
    service: `engine-api:${health.database.status}`,
    timestamp: new Date().toISOString(),
  };
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

export async function getNotifications(
  profileId?: string,
  limit: number = 20,
): Promise<AppNotification[]> {
  const resolvedId = profileId ?? readStoredProfileId() ?? undefined;
  if (!resolvedId) {
    return [];
  }

  const response = await request<EngineNotificationsResponse>(
    `/api/v1/notifications?profile_id=${encodeURIComponent(
      resolvedId,
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
  const resolvedId = profileId ?? readStoredProfileId() ?? undefined;
  if (!resolvedId) {
    return 0;
  }

  const response = await request<EngineUnreadNotificationsCountResponse>(
    `/api/v1/notifications/unread-count?profile_id=${encodeURIComponent(
      resolvedId,
    )}`,
  );

  return response.unread_count;
}

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
