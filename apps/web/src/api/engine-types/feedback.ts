import type { CompanyFeedbackStatus } from '@job-copilot/shared';

export type EngineJobFeedbackState = {
  saved: boolean;
  hidden: boolean;
  bad_fit: boolean;
  company_status?: CompanyFeedbackStatus | null;
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
