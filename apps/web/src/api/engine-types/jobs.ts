import type { EngineJobFeedbackState } from './feedback';

export type EngineJobPrimaryVariant = {
  source: string;
  source_job_id: string;
  source_url: string;
  fetched_at: string;
  last_seen_at: string;
  is_active: boolean;
  inactivated_at?: string | null;
};

export type EngineJobPresentation = {
  title: string;
  company: string;
  summary?: string | null;
  summary_quality?: string | null;
  summary_fallback: boolean;
  description_quality: string;
  location_label?: string | null;
  work_mode_label?: string | null;
  source_label?: string | null;
  outbound_url?: string | null;
  salary_label?: string | null;
  freshness_label?: string | null;
  badges: string[];
};

export type EngineJob = {
  id: string;
  title: string;
  company_name: string;
  description_text: string;
  location?: string | null;
  remote_type?: string | null;
  seniority?: string | null;
  salary_min?: number | null;
  salary_max?: number | null;
  salary_currency?: string | null;
  posted_at: string | null;
  first_seen_at: string;
  last_seen_at: string;
  is_active: boolean;
  inactivated_at?: string | null;
  reactivated_at?: string | null;
  lifecycle_stage: 'active' | 'inactive' | 'reactivated';
  primary_variant?: EngineJobPrimaryVariant | null;
  presentation: EngineJobPresentation;
  feedback: EngineJobFeedbackState;
};

export type EngineJobFeedSummary = {
  total_jobs: number;
  active_jobs: number;
  inactive_jobs: number;
  reactivated_jobs: number;
};

export type EngineRecentJobsResponse = {
  jobs: EngineJob[];
  summary: EngineJobFeedSummary;
};
