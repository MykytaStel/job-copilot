import type {
  CompanyFeedbackStatus,
  JobFeedbackReason,
  LegitimacySignal,
  SalaryFeedbackSignal,
  WorkModeFeedbackSignal,
} from '@job-copilot/shared/feedback';

export type EngineJobFeedbackState = {
  saved: boolean;
  hidden: boolean;
  bad_fit: boolean;
  company_status?: CompanyFeedbackStatus | null;
  salary_signal?: SalaryFeedbackSignal | null;
  interest_rating?: number | null;
  work_mode_signal?: WorkModeFeedbackSignal | null;
  legitimacy_signal?: LegitimacySignal | null;
  tags?: JobFeedbackReason[] | null;
};

export type EngineJobFeedbackRecord = {
  job_id: string;
  saved: boolean;
  hidden: boolean;
  bad_fit: boolean;
  updated_at: string;
};

export type EngineCompanyFeedbackRecord = {
  company_name: string;
  normalized_company_name: string;
  status: CompanyFeedbackStatus;
  notes: string;
  updated_at: string;
};

export type EngineFeedbackOverviewResponse = {
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

export type EngineFeedbackStatsResponse = {
  saved_this_week_count: number;
  hidden_this_week_count: number;
  bad_fit_this_week_count: number;
  whitelisted_companies_count: number;
  blacklisted_companies_count: number;
};

export type EngineFeedbackTimelineItem = {
  id: string;
  event_type: string;
  job_id?: string | null;
  job_title: string;
  company_name: string;
  reason?: string | null;
  created_at: string;
};

export type EngineFeedbackTimelineResponse = {
  profile_id: string;
  items: EngineFeedbackTimelineItem[];
  limit: number;
  offset: number;
  total_count: number;
  next_offset?: number | null;
};
