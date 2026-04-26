import { request } from './client';

type EngineJobsBySourceEntry = {
  source: string;
  count: number;
};

type EngineJobsByLifecycle = {
  total: number;
  active: number;
  inactive: number;
  reactivated: number;
};

type EngineFeedbackSummarySection = {
  saved_jobs_count: number;
  hidden_jobs_count: number;
  bad_fit_jobs_count: number;
  whitelisted_companies_count: number;
  blacklisted_companies_count: number;
};

type EngineFunnelSourceCountEntry = {
  source: string;
  count: number;
};

type EngineFunnelConversionRates = {
  open_rate_from_impressions: number;
  save_rate_from_opens: number;
  application_rate_from_saves: number;
};

type EngineSearchQualitySummary = {
  low_evidence_jobs: number;
  weak_description_jobs: number;
  role_mismatch_jobs: number;
  seniority_mismatch_jobs: number;
  source_mismatch_jobs: number;
  top_missing_signals: string[];
};

type EngineAnalyticsSummaryResponse = {
  profile_id: string;
  feedback: EngineFeedbackSummarySection;
  jobs_by_source: EngineJobsBySourceEntry[];
  jobs_by_lifecycle: EngineJobsByLifecycle;
  top_matched_roles: string[];
  top_matched_skills: string[];
  top_matched_keywords: string[];
  search_quality: EngineSearchQualitySummary;
};

type EngineFunnelSummaryResponse = {
  profile_id: string;
  impression_count: number;
  open_count: number;
  save_count: number;
  hide_count: number;
  bad_fit_count: number;
  application_created_count: number;
  fit_explanation_requested_count: number;
  application_coach_requested_count: number;
  cover_letter_draft_requested_count: number;
  interview_prep_requested_count: number;
  conversion_rates: EngineFunnelConversionRates;
  impressions_by_source: EngineFunnelSourceCountEntry[];
  opens_by_source: EngineFunnelSourceCountEntry[];
  saves_by_source: EngineFunnelSourceCountEntry[];
  applications_by_source: EngineFunnelSourceCountEntry[];
};

type EngineBehaviorSignalCountResponse = {
  key: string;
  save_count: number;
  hide_count: number;
  bad_fit_count: number;
  application_created_count: number;
  positive_count: number;
  negative_count: number;
  net_score: number;
};

type EngineBehaviorSummaryResponse = {
  profile_id: string;
  search_run_count: number;
  top_positive_sources: EngineBehaviorSignalCountResponse[];
  top_negative_sources: EngineBehaviorSignalCountResponse[];
  top_positive_role_families: EngineBehaviorSignalCountResponse[];
  top_negative_role_families: EngineBehaviorSignalCountResponse[];
  source_signal_counts: EngineBehaviorSignalCountResponse[];
  role_family_signal_counts: EngineBehaviorSignalCountResponse[];
};

type EngineLlmContextAnalyzedProfile = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

type EngineLlmContextEvidenceEntry = {
  type: string;
  label: string;
};

type EngineLlmContextResponse = {
  profile_id: string;
  analyzed_profile: EngineLlmContextAnalyzedProfile | null;
  profile_skills: string[];
  profile_keywords: string[];
  jobs_feed_summary: EngineJobsByLifecycle;
  feedback_summary: EngineFeedbackSummarySection;
  top_positive_evidence: EngineLlmContextEvidenceEntry[];
  top_negative_evidence: EngineLlmContextEvidenceEntry[];
};

type EngineProfileMlState = {
  profile_id: string;
  last_retrained_at: string | null;
  examples_since_retrain: number;
  last_artifact_version: string | null;
  last_training_status: string | null;
};

type EngineProfileMlMetricRecord = {
  id: string;
  profile_id: string;
  retrained_at: string;
  status: string;
  artifact_version: string | null;
  model_type: string | null;
  reason: string | null;
  metrics: Record<string, unknown> | null;
  training: Record<string, unknown> | null;
  feature_importances: Record<string, number> | null;
  benchmark: Record<string, unknown> | null;
};

type EngineRerankerMetricsResponse = {
  profile_id: string;
  state: EngineProfileMlState;
  summary: {
    run_count: number;
    trained_run_count: number;
    skipped_run_count: number;
    failed_run_count: number;
    warning_run_count: number;
    last_warning_reason: string | null;
  };
  runs: EngineProfileMlMetricRecord[];
};

export type JobsBySourceEntry = {
  source: string;
  count: number;
};

export type JobsByLifecycle = {
  total: number;
  active: number;
  inactive: number;
  reactivated: number;
};

export type AnalyticsFeedbackSummary = {
  savedJobsCount: number;
  hiddenJobsCount: number;
  badFitJobsCount: number;
  whitelistedCompaniesCount: number;
  blacklistedCompaniesCount: number;
};

export type AnalyticsSummary = {
  profileId: string;
  feedback: AnalyticsFeedbackSummary;
  jobsBySource: JobsBySourceEntry[];
  jobsByLifecycle: JobsByLifecycle;
  topMatchedRoles: string[];
  topMatchedSkills: string[];
  topMatchedKeywords: string[];
  searchQuality: {
    lowEvidenceJobs: number;
    weakDescriptionJobs: number;
    roleMismatchJobs: number;
    seniorityMismatchJobs: number;
    sourceMismatchJobs: number;
    topMissingSignals: string[];
  };
};

export type BehaviorSignalCount = {
  key: string;
  saveCount: number;
  hideCount: number;
  badFitCount: number;
  applicationCreatedCount: number;
  positiveCount: number;
  negativeCount: number;
  netScore: number;
};

export type BehaviorSummary = {
  profileId: string;
  searchRunCount: number;
  topPositiveSources: BehaviorSignalCount[];
  topNegativeSources: BehaviorSignalCount[];
  topPositiveRoleFamilies: BehaviorSignalCount[];
  topNegativeRoleFamilies: BehaviorSignalCount[];
  sourceSignalCounts: BehaviorSignalCount[];
  roleFamilySignalCounts: BehaviorSignalCount[];
};

export type FunnelSourceCountEntry = {
  source: string;
  count: number;
};

export type FunnelConversionRates = {
  openRateFromImpressions: number;
  saveRateFromOpens: number;
  applicationRateFromSaves: number;
};

export type FunnelSummary = {
  profileId: string;
  impressionCount: number;
  openCount: number;
  saveCount: number;
  hideCount: number;
  badFitCount: number;
  applicationCreatedCount: number;
  fitExplanationRequestedCount: number;
  applicationCoachRequestedCount: number;
  coverLetterDraftRequestedCount: number;
  interviewPrepRequestedCount: number;
  conversionRates: FunnelConversionRates;
  impressionsBySource: FunnelSourceCountEntry[];
  opensBySource: FunnelSourceCountEntry[];
  savesBySource: FunnelSourceCountEntry[];
  applicationsBySource: FunnelSourceCountEntry[];
};

export type LlmContextEvidenceEntry = {
  type: string;
  label: string;
};

export type LlmContextAnalyzedProfile = {
  summary: string;
  primaryRole: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

export type LlmContext = {
  profileId: string;
  analyzedProfile: LlmContextAnalyzedProfile | null;
  profileSkills: string[];
  profileKeywords: string[];
  jobsFeedSummary: JobsByLifecycle;
  feedbackSummary: AnalyticsFeedbackSummary;
  topPositiveEvidence: LlmContextEvidenceEntry[];
  topNegativeEvidence: LlmContextEvidenceEntry[];
};

export type RerankerMlState = {
  profileId: string;
  lastRetrainedAt: string | null;
  examplesSinceRetrain: number;
  lastArtifactVersion: string | null;
  lastTrainingStatus: string | null;
};

export type RerankerMetricRecord = {
  id: string;
  profileId: string;
  retrainedAt: string;
  status: string;
  artifactVersion: string | null;
  modelType: string | null;
  reason: string | null;
  metrics: Record<string, unknown> | null;
  training: Record<string, unknown> | null;
  featureImportances: Record<string, number> | null;
  benchmark: Record<string, unknown> | null;
};

export type RerankerMetrics = {
  profileId: string;
  state: RerankerMlState;
  summary: {
    runCount: number;
    trainedRunCount: number;
    skippedRunCount: number;
    failedRunCount: number;
    warningRunCount: number;
    lastWarningReason: string | null;
  };
  runs: RerankerMetricRecord[];
};

function mapFeedbackSummarySection(
  summary: EngineFeedbackSummarySection,
): AnalyticsFeedbackSummary {
  return {
    savedJobsCount: summary.saved_jobs_count,
    hiddenJobsCount: summary.hidden_jobs_count,
    badFitJobsCount: summary.bad_fit_jobs_count,
    whitelistedCompaniesCount: summary.whitelisted_companies_count,
    blacklistedCompaniesCount: summary.blacklisted_companies_count,
  };
}

function mapJobsByLifecycle(lifecycle: EngineJobsByLifecycle): JobsByLifecycle {
  return {
    total: lifecycle.total,
    active: lifecycle.active,
    inactive: lifecycle.inactive,
    reactivated: lifecycle.reactivated,
  };
}

function mapBehaviorSignalCount(signal: EngineBehaviorSignalCountResponse): BehaviorSignalCount {
  return {
    key: signal.key,
    saveCount: signal.save_count,
    hideCount: signal.hide_count,
    badFitCount: signal.bad_fit_count,
    applicationCreatedCount: signal.application_created_count,
    positiveCount: signal.positive_count,
    negativeCount: signal.negative_count,
    netScore: signal.net_score,
  };
}

function mapRerankerMetrics(response: EngineRerankerMetricsResponse): RerankerMetrics {
  return {
    profileId: response.profile_id,
    state: {
      profileId: response.state.profile_id,
      lastRetrainedAt: response.state.last_retrained_at,
      examplesSinceRetrain: response.state.examples_since_retrain,
      lastArtifactVersion: response.state.last_artifact_version,
      lastTrainingStatus: response.state.last_training_status,
    },
    summary: {
      runCount: response.summary.run_count,
      trainedRunCount: response.summary.trained_run_count,
      skippedRunCount: response.summary.skipped_run_count,
      failedRunCount: response.summary.failed_run_count,
      warningRunCount: response.summary.warning_run_count,
      lastWarningReason: response.summary.last_warning_reason,
    },
    runs: response.runs.map((run) => ({
      id: run.id,
      profileId: run.profile_id,
      retrainedAt: run.retrained_at,
      status: run.status,
      artifactVersion: run.artifact_version,
      modelType: run.model_type,
      reason: run.reason,
      metrics: run.metrics,
      training: run.training,
      featureImportances: run.feature_importances,
      benchmark: run.benchmark,
    })),
  };
}

export async function getAnalyticsSummary(profileId: string): Promise<AnalyticsSummary> {
  const response = await request<EngineAnalyticsSummaryResponse>(
    `/api/v1/profiles/${profileId}/analytics/summary`,
  );

  return {
    profileId: response.profile_id,
    feedback: mapFeedbackSummarySection(response.feedback),
    jobsBySource: response.jobs_by_source,
    jobsByLifecycle: mapJobsByLifecycle(response.jobs_by_lifecycle),
    topMatchedRoles: response.top_matched_roles,
    topMatchedSkills: response.top_matched_skills,
    topMatchedKeywords: response.top_matched_keywords,
    searchQuality: {
      lowEvidenceJobs: response.search_quality.low_evidence_jobs,
      weakDescriptionJobs: response.search_quality.weak_description_jobs,
      roleMismatchJobs: response.search_quality.role_mismatch_jobs,
      seniorityMismatchJobs: response.search_quality.seniority_mismatch_jobs,
      sourceMismatchJobs: response.search_quality.source_mismatch_jobs,
      topMissingSignals: response.search_quality.top_missing_signals,
    },
  };
}

export async function getBehaviorSummary(profileId: string): Promise<BehaviorSummary> {
  const response = await request<EngineBehaviorSummaryResponse>(
    `/api/v1/profiles/${profileId}/behavior-summary`,
  );

  return {
    profileId: response.profile_id,
    searchRunCount: response.search_run_count,
    topPositiveSources: response.top_positive_sources.map(mapBehaviorSignalCount),
    topNegativeSources: response.top_negative_sources.map(mapBehaviorSignalCount),
    topPositiveRoleFamilies: response.top_positive_role_families.map(mapBehaviorSignalCount),
    topNegativeRoleFamilies: response.top_negative_role_families.map(mapBehaviorSignalCount),
    sourceSignalCounts: response.source_signal_counts.map(mapBehaviorSignalCount),
    roleFamilySignalCounts: response.role_family_signal_counts.map(mapBehaviorSignalCount),
  };
}

export async function getFunnelSummary(profileId: string): Promise<FunnelSummary> {
  const response = await request<EngineFunnelSummaryResponse>(
    `/api/v1/profiles/${profileId}/funnel-summary`,
  );

  return {
    profileId: response.profile_id,
    impressionCount: response.impression_count,
    openCount: response.open_count,
    saveCount: response.save_count,
    hideCount: response.hide_count,
    badFitCount: response.bad_fit_count,
    applicationCreatedCount: response.application_created_count,
    fitExplanationRequestedCount: response.fit_explanation_requested_count,
    applicationCoachRequestedCount: response.application_coach_requested_count,
    coverLetterDraftRequestedCount: response.cover_letter_draft_requested_count,
    interviewPrepRequestedCount: response.interview_prep_requested_count,
    conversionRates: {
      openRateFromImpressions: response.conversion_rates.open_rate_from_impressions,
      saveRateFromOpens: response.conversion_rates.save_rate_from_opens,
      applicationRateFromSaves: response.conversion_rates.application_rate_from_saves,
    },
    impressionsBySource: response.impressions_by_source,
    opensBySource: response.opens_by_source,
    savesBySource: response.saves_by_source,
    applicationsBySource: response.applications_by_source,
  };
}

export async function getLlmContext(profileId: string): Promise<LlmContext> {
  const response = await request<EngineLlmContextResponse>(
    `/api/v1/profiles/${profileId}/analytics/llm-context`,
  );

  return {
    profileId: response.profile_id,
    analyzedProfile: response.analyzed_profile
      ? {
          summary: response.analyzed_profile.summary,
          primaryRole: response.analyzed_profile.primary_role,
          seniority: response.analyzed_profile.seniority,
          skills: response.analyzed_profile.skills,
          keywords: response.analyzed_profile.keywords,
        }
      : null,
    profileSkills: response.profile_skills,
    profileKeywords: response.profile_keywords,
    jobsFeedSummary: mapJobsByLifecycle(response.jobs_feed_summary),
    feedbackSummary: mapFeedbackSummarySection(response.feedback_summary),
    topPositiveEvidence: response.top_positive_evidence,
    topNegativeEvidence: response.top_negative_evidence,
  };
}

export async function getRerankerMetrics(profileId: string): Promise<RerankerMetrics> {
  const response = await request<EngineRerankerMetricsResponse>(
    `/api/v1/profiles/${profileId}/reranker/metrics`,
  );

  return mapRerankerMetrics(response);
}

type EngineIngestionStatsResponse = {
  last_ingested_at: string | null;
  total_jobs: number;
  active_jobs: number;
};

export type IngestionStats = {
  lastIngestedAt: string | null;
  totalJobs: number;
  activeJobs: number;
};

export async function getIngestionStats(): Promise<IngestionStats> {
  const response = await request<EngineIngestionStatsResponse>('/api/v1/ingestion/stats');

  return {
    lastIngestedAt: response.last_ingested_at,
    totalJobs: response.total_jobs,
    activeJobs: response.active_jobs,
  };
}
