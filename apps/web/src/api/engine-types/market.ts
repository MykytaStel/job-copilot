export type InternalMarketTrend = 'up' | 'down' | 'stable';

export type EngineMarketOverview = {
  new_jobs_this_week: number;
  active_companies_count: number;
  active_jobs_count: number;
  remote_percentage: number;
};

export type EngineMarketCompanyEntry = {
  company_name: string;
  active_jobs: number;
  this_week: number;
  prev_week: number;
  velocity: number;
};

export type EngineMarketCompaniesResponse = {
  companies: EngineMarketCompanyEntry[];
};

export type EngineMarketSalaryTrend = {
  seniority: string;
  p25: number;
  median: number;
  p75: number;
  sample_count: number;
};

export type EngineMarketRoleDemandEntry = {
  role_group: string;
  this_period: number;
  prev_period: number;
  trend: InternalMarketTrend;
};
