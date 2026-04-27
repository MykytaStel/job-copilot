import type { JobFeedbackState } from './feedback';

export type JobSource = 'manual' | 'url' | 'rss';

export interface JobPostingInput {
  source: JobSource;
  url?: string;
  rawText?: string;
  title?: string;
  company?: string;
}

export interface JobSourceVariant {
  source: string;
  sourceUrl: string;
  fetchedAt: string;
  lastSeenAt: string;
  isActive: boolean;
  inactivatedAt?: string;
}

export interface JobScoreSignal { label: string; delta: number; }

export interface JobPresentation {
  title: string;
  company: string;
  summary?: string;
  summaryQuality?: string;
  summaryFallback?: boolean;
  descriptionQuality?: string;
  locationLabel?: string;
  workModeLabel?: string;
  sourceLabel?: string;
  outboundUrl?: string;
  salaryLabel?: string;
  freshnessLabel?: string;
  lifecyclePrimaryLabel?: string;
  lifecycleSecondaryLabel?: string;
  badges: string[]; scoreSignals?: JobScoreSignal[];
}

export interface JobPosting {
  id: string;
  source: JobSource;
  url?: string;
  title: string;
  company: string;
  description: string;
  location?: string;
  notes: string;
  createdAt: string;
  postedAt?: string;
  firstSeenAt?: string;
  lastSeenAt?: string;
  isActive?: boolean;
  inactivatedAt?: string;
  reactivatedAt?: string;
  lifecycleStage?: 'active' | 'inactive' | 'reactivated';
  primaryVariant?: JobSourceVariant;
  salaryMin?: number;
  salaryMax?: number;
  salaryCurrency?: string;
  seniority?: string;
  remoteType?: string;
  presentation?: JobPresentation;
  feedback?: JobFeedbackState;
}

export interface JobFeedSummary {
  totalJobs: number;
  activeJobs: number;
  inactiveJobs: number;
  reactivatedJobs: number;
}

export interface MatchResult {
  id: string;
  jobId: string;
  resumeId: string;
  /** 0-100 overall fit percentage */
  score: number;
  matchedSkills: string[];
  missingSkills: string[];
  notes: string;
  createdAt: string;
}

export interface JobAlert {
  id: string;
  /** Keywords to match against job title + description (OR logic) */
  keywords: string[];
  telegramChatId: string;
  active: boolean;
  createdAt: string;
}

export interface JobAlertInput {
  keywords: string[];
  telegramChatId: string;
}

export interface ImportResult {
  url: string;
  status: 'imported' | 'duplicate' | 'error';
  job?: JobPosting;
  error?: string;
}

export interface ImportBatchResponse {
  results: ImportResult[];
}

export interface BackupMeta {
  version: string;
  exportedAt: string;
}
