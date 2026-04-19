import type {
  HealthResponse,
  JobFeedbackState,
  JobPosting,
} from '@job-copilot/shared';
import {
  mlRequest,
  request,
} from './api/client';
import type {
  EngineHealthResponse,
} from './api/engine-types';
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

export {
  getFeedback,
  markJobSaved,
  hideJobForProfile,
  markJobBadFit,
  unsaveJob,
  unhideJob,
  unmarkJobBadFit,
  addCompanyWhitelist,
  removeCompanyWhitelist,
  addCompanyBlacklist,
  removeCompanyBlacklist,
} from './api/feedback';

export {
  getNotifications,
  markNotificationRead,
  getUnreadCount,
} from './api/notifications';
export type {
  AppNotificationType,
  AppNotification,
} from './api/notifications';

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

export async function getHealth(): Promise<HealthResponse> {
  const health = await request<EngineHealthResponse>('/health');

  return {
    status: 'ok',
    service: `engine-api:${health.database.status}`,
    timestamp: new Date().toISOString(),
  };
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
