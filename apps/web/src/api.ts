import type { HealthResponse } from '@job-copilot/shared';
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
