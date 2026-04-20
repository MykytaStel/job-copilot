export type CompanyFeedbackStatus = 'whitelist' | 'blacklist';

export interface JobFeedbackState {
  saved: boolean;
  hidden: boolean;
  badFit: boolean;
  companyStatus?: CompanyFeedbackStatus;
}

export interface JobFeedbackRecord {
  jobId: string;
  saved: boolean;
  hidden: boolean;
  badFit: boolean;
  updatedAt: string;
}

export interface CompanyFeedbackRecord {
  companyName: string;
  normalizedCompanyName: string;
  status: CompanyFeedbackStatus;
  updatedAt: string;
}

export interface FeedbackSummary {
  savedJobsCount: number;
  hiddenJobsCount: number;
  badFitJobsCount: number;
  whitelistedCompaniesCount: number;
  blacklistedCompaniesCount: number;
}

export interface FeedbackOverview {
  profileId: string;
  jobs: JobFeedbackRecord[];
  companies: CompanyFeedbackRecord[];
  summary: FeedbackSummary;
}
