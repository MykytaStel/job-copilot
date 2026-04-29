import type { HealthResponse } from '@job-copilot/shared/health';
import { request } from './api/client';
import type { EngineHealthResponse } from './api/engine-types';

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
} from './api/applications';
export type { GlobalSearchApplicationResult } from './api/applications';

export { getContacts, createContact, linkContact } from './api/contacts';

export {
  getMarketOverview,
  getMarketCompanies,
  getMarketSalaries,
  getMarketRoles,
} from './api/market';
export type {
  SkillStat,
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
  getFeedbackStats,
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

export { getNotifications, markNotificationRead, getUnreadCount } from './api/notifications';
export type { AppNotificationType, AppNotification } from './api/notifications';

export {
  getProfileInsights,
  getWeeklyGuidance,
  getJobFitExplanation,
  getApplicationCoach,
  getCoverLetterDraft,
  getInterviewPrep,
} from './api/enrichment';
export type {
  ProfileInsights,
  JobFitExplanation,
  ApplicationCoach,
  CoverLetterDraft,
  InterviewPrep,
  WeeklyGuidance,
  WeeklyGuidanceRequest,
  JobFitExplanationRequest,
  ApplicationCoachRequest,
  CoverLetterDraftRequest,
  InterviewPrepRequest,
} from './api/enrichment';

export async function getHealth(): Promise<HealthResponse> {
  const health = await request<EngineHealthResponse>('/health');

  return {
    status: 'ok',
    service: `engine-api:${health.database.status}`,
    timestamp: new Date().toISOString(),
  };
}
