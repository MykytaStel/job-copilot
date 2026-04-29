export type InternalMarketTrend = 'up' | 'down' | 'stable';

export type EngineMarketOverview = {
  new_jobs_this_week: number;
  active_companies_count: number;
  active_jobs_count: number;
  remote_percentage: number;
};

export type EngineMarketCompanyEntry = {
  company_name: string;
  normalized_company_name?: string;
  active_jobs: number;
  this_week: number;
  prev_week: number;
  velocity: number;
  sources?: string[];
  top_role_groups?: string[];
  latest_job_ids?: string[];
  data_quality_flags?: string[];
};

export type EngineMarketCompaniesResponse = {
  companies: EngineMarketCompanyEntry[];
};

export type EngineMarketCompanyVelocityTrend = 'growing' | 'stable' | 'declining';

export type EngineMarketCompanyVelocityEntry = {
  company: string;
  job_count: number;
  trend: EngineMarketCompanyVelocityTrend;
};

export type EngineMarketFreezeSignalEntry = {
  company: string;
  last_posted_at: string;
  days_since_last_post: number;
  historical_count: number;
};

export type EngineMarketSalaryTrend = {
  seniority: string;
  currency: string;
  p25: number;
  median: number;
  p75: number;
  sample_count: number;
};

export type EngineMarketSalaryBySeniorityEntry = {
  seniority: string;
  median_min: number;
  median_max: number;
  sample_size: number;
};

export type EngineMarketRoleDemandEntry = {
  role_group: string;
  this_period: number;
  prev_period: number;
  trend: InternalMarketTrend;
};

export type EngineMarketRegionDemandEntry = {
  region: string;
  job_count: number;
  top_roles: string[];
};

export type EngineMarketTechDemandEntry = {
  skill: string;
  job_count: number;
  percentage: number;
};
