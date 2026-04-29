export type CompanyFeedbackStatus = 'whitelist' | 'blacklist';

export type SalaryFeedbackSignal =
  | 'above_expectation'
  | 'at_expectation'
  | 'below_expectation'
  | 'not_shown';

export type WorkModeFeedbackSignal =
  | 'matches_preference'
  | 'would_accept'
  | 'deal_breaker';

export type LegitimacySignal =
  | 'looks_real'
  | 'suspicious'
  | 'spam'
  | 'duplicate';

export type JobFeedbackReason =
  | 'salary_too_low'
  | 'not_remote'
  | 'too_junior'
  | 'too_senior'
  | 'bad_tech_stack'
  | 'suspicious_posting'
  | 'already_applied'
  | 'duplicate_posting'
  | 'bad_company_rep'
  | 'wrong_city'
  | 'wrong_industry'
  | 'visa_sponsorship_required'
  | 'interesting_challenge'
  | 'great_company'
  | 'good_salary'
  | 'remote_ok'
  | 'good_tech_stack'
  | 'fast_growth_company'
  | 'nice_title';

export interface JobFeedbackState {
  saved: boolean;
  hidden: boolean;
  badFit: boolean;
  companyStatus?: CompanyFeedbackStatus;
  salarySignal?: SalaryFeedbackSignal;
  interestRating?: number;
  workModeSignal?: WorkModeFeedbackSignal;
  legitimacySignal?: LegitimacySignal;
  tags?: JobFeedbackReason[];
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
  notes: string;
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
