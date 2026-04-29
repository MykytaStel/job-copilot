import { request } from './client';
import type {
  EngineMarketCompaniesResponse,
  EngineMarketCompanyVelocityEntry,
  EngineMarketCompanyVelocityTrend,
  EngineMarketFreezeSignalEntry,
  EngineMarketOverview,
  EngineMarketRegionDemandEntry,
  EngineMarketRoleDemandEntry,
  EngineMarketSalaryBySeniorityEntry,
  EngineMarketSalaryTrend,
  EngineMarketTechDemandEntry,
} from './engine-types';

export interface SkillStat {
  skill: string;
  count: number;
  pct: number;
  inResume: boolean;
}

export interface MarketInsights {
  totalJobs: number;
  coverageScore: number;
  topSkills: SkillStat[];
  hotGaps: string[];
  salaryMentions: string[];
}

export type MarketTrend = 'up' | 'down' | 'stable';
export type MarketCompanyVelocityTrend = EngineMarketCompanyVelocityTrend;

export type MarketOverview = {
  newJobsThisWeek: number;
  activeCompaniesCount: number;
  activeJobsCount: number;
  remotePercentage: number;
};

export type MarketCompany = {
  companyName: string;
  normalizedCompanyName: string;
  activeJobs: number;
  thisWeek: number;
  prevWeek: number;
  velocity: number;
  sources: string[];
  topRoleGroups: string[];
  latestJobIds: string[];
  dataQualityFlags: string[];
};

export type MarketCompanyVelocity = {
  company: string;
  jobCount: number;
  trend: MarketCompanyVelocityTrend;
};

export type MarketFreezeSignal = {
  company: string;
  lastPostedAt: string;
  daysSinceLastPost: number;
  historicalCount: number;
};

export type MarketSalaryTrend = {
  seniority: string;
  currency: string;
  p25: number;
  median: number;
  p75: number;
  sampleCount: number;
};

export type MarketSalaryBySeniority = {
  seniority: string;
  medianMin: number;
  medianMax: number;
  sampleSize: number;
};

export type MarketRoleDemand = {
  roleGroup: string;
  thisPeriod: number;
  prevPeriod: number;
  trend: MarketTrend;
};

export type MarketRegionDemand = {
  region: string;
  jobCount: number;
  topRoles: string[];
};

export type MarketTechDemand = {
  skill: string;
  jobCount: number;
  percentage: number;
};

const DEFAULT_SENIORITY_BUCKETS = ['junior', 'middle', 'senior', 'lead'] as const;

export async function getMarketOverview(): Promise<MarketOverview> {
  const response = await request<EngineMarketOverview>('/api/v1/market/overview');

  return {
    newJobsThisWeek: response.new_jobs_this_week,
    activeCompaniesCount: response.active_companies_count,
    activeJobsCount: response.active_jobs_count,
    remotePercentage: response.remote_percentage,
  };
}

export async function getMarketCompanies(limit = 10): Promise<MarketCompany[]> {
  const response = await request<EngineMarketCompaniesResponse>(
    `/api/v1/market/companies?limit=${encodeURIComponent(String(limit))}`,
  );

  return response.companies.map((company) => ({
    companyName: company.company_name,
    normalizedCompanyName:
      company.normalized_company_name ?? company.company_name.trim().toLowerCase(),
    activeJobs: company.active_jobs,
    thisWeek: company.this_week,
    prevWeek: company.prev_week,
    velocity: company.velocity,
    sources: company.sources ?? [],
    topRoleGroups: company.top_role_groups ?? [],
    latestJobIds: company.latest_job_ids ?? [],
    dataQualityFlags: company.data_quality_flags ?? [],
  }));
}

export async function getMarketCompanyVelocity(): Promise<MarketCompanyVelocity[]> {
  const response = await request<EngineMarketCompanyVelocityEntry[]>(
    '/api/v1/market/company-velocity',
  );

  return response.map((entry) => ({
    company: entry.company,
    jobCount: entry.job_count,
    trend: entry.trend,
  }));
}

export async function getMarketFreezeSignals(): Promise<MarketFreezeSignal[]> {
  const response = await request<EngineMarketFreezeSignalEntry[]>('/api/v1/market/freeze-signals');

  return response.map((entry) => ({
    company: entry.company,
    lastPostedAt: entry.last_posted_at,
    daysSinceLastPost: entry.days_since_last_post,
    historicalCount: entry.historical_count,
  }));
}

export async function getMarketSalaries(
  seniorityBuckets: readonly string[] = DEFAULT_SENIORITY_BUCKETS,
): Promise<MarketSalaryTrend[]> {
  const buckets = Array.from(
    new Set(seniorityBuckets.map((bucket) => bucket.trim().toLowerCase()).filter(Boolean)),
  );

  const response = await request<EngineMarketSalaryTrend[]>('/api/v1/market/salary-trends');

  const trendsBySeniority = new Map(
    response.map((trend) => [
      trend.seniority.toLowerCase(),
      {
        seniority: trend.seniority,
        currency: trend.currency,
        p25: trend.p25,
        median: trend.median,
        p75: trend.p75,
        sampleCount: trend.sample_count,
      } satisfies MarketSalaryTrend,
    ]),
  );

  return buckets
    .map((bucket) => trendsBySeniority.get(bucket))
    .filter((result): result is MarketSalaryTrend => result !== undefined);
}

export async function getMarketSalaryBySeniority(): Promise<MarketSalaryBySeniority[]> {
  const response = await request<EngineMarketSalaryBySeniorityEntry[]>(
    '/api/v1/market/salary-by-seniority',
  );

  return response.map((entry) => ({
    seniority: entry.seniority,
    medianMin: entry.median_min,
    medianMax: entry.median_max,
    sampleSize: entry.sample_size,
  }));
}

export async function getMarketRoles(period = 30): Promise<MarketRoleDemand[]> {
  const response = await request<EngineMarketRoleDemandEntry[]>(
    `/api/v1/market/roles?period=${encodeURIComponent(String(period))}`,
  );

  return response.map((entry) => ({
    roleGroup: entry.role_group,
    thisPeriod: entry.this_period,
    prevPeriod: entry.prev_period,
    trend: entry.trend,
  }));
}

export async function getMarketRegionBreakdown(): Promise<MarketRegionDemand[]> {
  const response = await request<EngineMarketRegionDemandEntry[]>(
    '/api/v1/market/region-breakdown',
  );

  return response.map((entry) => ({
    region: entry.region,
    jobCount: entry.job_count,
    topRoles: entry.top_roles ?? [],
  }));
}

export async function getMarketTechDemand(): Promise<MarketTechDemand[]> {
  const response = await request<EngineMarketTechDemandEntry[]>('/api/v1/market/tech-demand');

  return response.map((entry) => ({
    skill: entry.skill,
    jobCount: entry.job_count,
    percentage: entry.percentage,
  }));
}
