import type {
  ApplicationDetail as RootApplicationDetail,
  DashboardStats as RootDashboardStats,
  EngineAnalyzeProfileResponse as RootEngineAnalyzeProfileResponse,
  EngineBuildSearchProfileResponse as RootEngineBuildSearchProfileResponse,
  FeedbackOverview as RootFeedbackOverview,
  HealthResponse as RootHealthResponse,
  JobPosting as RootJobPosting,
} from '@job-copilot/shared';

import type { DashboardStats } from '@job-copilot/shared/analytics';
import type { ApplicationDetail } from '@job-copilot/shared/applications';
import type { FeedbackOverview } from '@job-copilot/shared/feedback';
import type { HealthResponse } from '@job-copilot/shared/health';
import type { JobPosting } from '@job-copilot/shared/jobs';
import type { EngineAnalyzeProfileResponse } from '@job-copilot/shared/profiles';
import type { EngineBuildSearchProfileResponse } from '@job-copilot/shared/search';

type IsEqual<Left, Right> = (<T>() => T extends Left ? 1 : 2) extends (
  <T>() => T extends Right ? 1 : 2
)
  ? true
  : false;

type Assert<T extends true> = T;

type JobPostingExportIsStable = Assert<IsEqual<RootJobPosting, JobPosting>>;
type FeedbackExportIsStable = Assert<IsEqual<RootFeedbackOverview, FeedbackOverview>>;
type ProfileExportIsStable = Assert<
  IsEqual<RootEngineAnalyzeProfileResponse, EngineAnalyzeProfileResponse>
>;
type SearchExportIsStable = Assert<
  IsEqual<RootEngineBuildSearchProfileResponse, EngineBuildSearchProfileResponse>
>;
type ApplicationExportIsStable = Assert<IsEqual<RootApplicationDetail, ApplicationDetail>>;
type AnalyticsExportIsStable = Assert<IsEqual<RootDashboardStats, DashboardStats>>;
type HealthExportIsStable = Assert<IsEqual<RootHealthResponse, HealthResponse>>;

export type PublicContractsExportChecks =
  | JobPostingExportIsStable
  | FeedbackExportIsStable
  | ProfileExportIsStable
  | SearchExportIsStable
  | ApplicationExportIsStable
  | AnalyticsExportIsStable
  | HealthExportIsStable;
