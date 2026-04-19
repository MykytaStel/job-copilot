import type {
  BackupMeta,
  ImportBatchResponse,
  JobAlert,
  JobAlertInput,
  JobFeedSummary,
  JobPosting,
  JobPostingInput,
  MatchResult,
  ResumeVersion,
} from '@job-copilot/shared';
import {
  RECENT_JOBS_LIMIT_MAX,
  json,
  readStoredProfileId,
  request,
  unsupported,
  unsupportedPromise,
  withProfileIdQuery,
} from './client';
import type {
  EngineFitExplanation,
  EngineGlobalSearchResponse,
  EngineJob,
  EngineMatchResult,
  EngineRecentJobsResponse,
  EngineRunSearchResponse,
} from './engine-types';
import {
  mapJob,
  mapJobFeedSummary,
  mapMatchResult,
  uniquePreservingOrder,
} from './mappers';
import type { SearchProfileBuildResult } from './profiles';
import type { GlobalSearchApplicationResult } from './applications';

export type RankedJob = {
  jobId: string;
  title: string;
  companyName: string;
  score: number;
  matchedTerms: string[];
  positiveReasons: string[];
  negativeReasons: string[];
  missingSignals: string[];
  descriptionQuality: string;
};

export type FitAnalysis = {
  profileId: string;
  jobId: string;
  score: number;
  matchedTerms: string[];
  matchedRoles: string[];
  matchedSkills: string[];
  matchedKeywords: string[];
  missingTerms: string[];
  descriptionQuality: string;
  positiveReasons: string[];
  negativeReasons: string[];
  evidence: string[];
};

export type SearchRunRequest = {
  searchProfile: SearchProfileBuildResult['searchProfile'];
  profileId?: string;
  limit?: number;
};

export type FitExplanation = {
  jobId: string;
  score: number;
  matchedRoles: string[];
  matchedSkills: string[];
  matchedKeywords: string[];
  missingSignals: string[];
  sourceMatch: boolean;
  workModeMatch?: boolean;
  regionMatch?: boolean;
  descriptionQuality: string;
  positiveReasons: string[];
  negativeReasons: string[];
  reasons: string[];
};

export type RankedJobResult = {
  job: JobPosting;
  source: string;
  fit: FitExplanation;
};

export type SearchRunResult = {
  results: RankedJobResult[];
  meta: {
    totalCandidates: number;
    filteredOutBySource: number;
    filteredOutHidden: number;
    filteredOutCompanyBlacklist: number;
    scoredJobs: number;
    returnedJobs: number;
    lowEvidenceJobs: number;
    weakDescriptionJobs: number;
    roleMismatchJobs: number;
    seniorityMismatchJobs: number;
    sourceMismatchJobs: number;
    topMissingSignals: string[];
  };
};

export type GlobalSearchResults = {
  jobs: JobPosting[];
  applications: GlobalSearchApplicationResult[];
};

export async function getJobs(): Promise<JobPosting[]> {
  const response = await request<EngineRecentJobsResponse>(
    withProfileIdQuery('/api/v1/jobs/recent'),
  );
  return response.jobs.map(mapJob);
}

export async function getJobsFeed(params?: {
  lifecycle?: string;
  source?: string;
  limit?: number;
}): Promise<{
  jobs: JobPosting[];
  summary: JobFeedSummary;
}> {
  const qs = new URLSearchParams();
  if (params?.lifecycle) qs.set('lifecycle', params.lifecycle);
  if (params?.source) qs.set('source', params.source);
  if (params?.limit) {
    qs.set('limit', String(Math.min(params.limit, RECENT_JOBS_LIMIT_MAX)));
  }
  const profileId = readStoredProfileId();
  if (profileId) qs.set('profile_id', profileId);
  const query = qs.toString();
  const response = await request<EngineRecentJobsResponse>(
    `/api/v1/jobs/recent${query ? `?${query}` : ''}`,
  );
  return {
    jobs: response.jobs.map(mapJob),
    summary: mapJobFeedSummary(response.summary),
  };
}

export async function getJob(id: string): Promise<JobPosting> {
  const job = await request<EngineJob>(withProfileIdQuery(`/api/v1/jobs/${id}`));
  return mapJob(job);
}

export async function runSearch(
  payload: SearchRunRequest,
): Promise<SearchRunResult> {
  const profileId = readStoredProfileId();
  const response = await request<EngineRunSearchResponse>(
    '/api/v1/search/run',
    json('POST', {
      profile_id: payload.profileId ?? profileId ?? undefined,
      search_profile: {
        primary_role: payload.searchProfile.primaryRole,
        primary_role_confidence: payload.searchProfile.primaryRoleConfidence,
        target_roles: payload.searchProfile.targetRoles,
        role_candidates: payload.searchProfile.roleCandidates,
        seniority: payload.searchProfile.seniority,
        target_regions: payload.searchProfile.targetRegions,
        work_modes: payload.searchProfile.workModes,
        allowed_sources: payload.searchProfile.allowedSources,
        profile_skills: payload.searchProfile.profileSkills,
        profile_keywords: payload.searchProfile.profileKeywords,
        search_terms: payload.searchProfile.searchTerms,
        exclude_terms: payload.searchProfile.excludeTerms,
      },
      limit: payload.limit,
    }),
  );

  return {
    results: response.results.map((result) => ({
      job: mapJob(result.job),
      source: result.job.primary_variant?.source ?? '',
      fit: {
        jobId: result.fit.job_id,
        score: result.fit.score,
        matchedRoles: result.fit.matched_roles,
        matchedSkills: result.fit.matched_skills,
        matchedKeywords: result.fit.matched_keywords,
        missingSignals: result.fit.missing_signals,
        sourceMatch: result.fit.source_match,
        workModeMatch: result.fit.work_mode_match ?? undefined,
        regionMatch: result.fit.region_match ?? undefined,
        descriptionQuality: result.fit.description_quality,
        positiveReasons: result.fit.positive_reasons,
        negativeReasons: result.fit.negative_reasons,
        reasons: result.fit.reasons,
      },
    })),
    meta: {
      totalCandidates: response.meta.total_candidates,
      filteredOutBySource: response.meta.filtered_out_by_source,
      filteredOutHidden: response.meta.filtered_out_hidden,
      filteredOutCompanyBlacklist: response.meta.filtered_out_company_blacklist,
      scoredJobs: response.meta.scored_jobs,
      returnedJobs: response.meta.returned_jobs,
      lowEvidenceJobs: response.meta.low_evidence_jobs,
      weakDescriptionJobs: response.meta.weak_description_jobs,
      roleMismatchJobs: response.meta.role_mismatch_jobs,
      seniorityMismatchJobs: response.meta.seniority_mismatch_jobs,
      sourceMismatchJobs: response.meta.source_mismatch_jobs,
      topMissingSignals: response.meta.top_missing_signals,
    },
  };
}

export async function rerankJobs(profileId: string, jobIds: string[]): Promise<RankedJob[]> {
  const uniqueJobIds = Array.from(
    new Set(jobIds.map((jobId) => jobId.trim()).filter(Boolean)),
  );
  if (uniqueJobIds.length === 0) {
    return [];
  }

  const response = await request<{
    profile_id: string;
    results: EngineFitExplanation[];
  }>(`/api/v1/profiles/${profileId}/jobs/match`, json('POST', {
    job_ids: uniqueJobIds,
  }));

  return response.results
    .map((fit) => ({
      jobId: fit.job_id,
      title: '',
      companyName: '',
      score: fit.score,
      matchedTerms: uniquePreservingOrder([
        ...fit.matched_roles,
        ...fit.matched_skills,
        ...fit.matched_keywords,
      ]),
      positiveReasons: fit.positive_reasons,
      negativeReasons: fit.negative_reasons,
      missingSignals: fit.missing_signals,
      descriptionQuality: fit.description_quality,
    }))
    .sort(
      (left, right) =>
        right.score - left.score ||
        left.jobId.localeCompare(right.jobId),
    );
}

export async function analyzeFit(profileId: string, jobId: string): Promise<FitAnalysis> {
  const result = await request<EngineFitExplanation>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/match`,
  );
  const matchedTerms = uniquePreservingOrder([
    ...result.matched_roles,
    ...result.matched_skills,
    ...result.matched_keywords,
  ]);
  const evidence = result.positive_reasons.length > 0 ? result.positive_reasons : result.reasons;

  return {
    profileId,
    jobId: result.job_id,
    score: result.score,
    matchedTerms,
    matchedRoles: result.matched_roles,
    matchedSkills: result.matched_skills,
    matchedKeywords: result.matched_keywords,
    missingTerms: result.missing_signals,
    descriptionQuality: result.description_quality,
    positiveReasons: result.positive_reasons,
    negativeReasons: result.negative_reasons,
    evidence,
  };
}

export async function runMatch(jobId: string): Promise<MatchResult> {
  const result = await request<EngineMatchResult>(
    `/api/v1/jobs/${jobId}/match`,
    json('POST', {}),
  );
  return mapMatchResult(result);
}

export async function getMatch(jobId: string): Promise<MatchResult> {
  const result = await request<EngineMatchResult>(`/api/v1/jobs/${jobId}/match`);
  return mapMatchResult(result);
}

export async function globalSearch(query: string): Promise<GlobalSearchResults> {
  const result = await request<EngineGlobalSearchResponse>(
    `/api/v1/search?q=${encodeURIComponent(query)}&limit=10`,
  );

  return {
    jobs: result.jobs.map(mapJob),
    applications: result.applications.map((application) => ({
      id: application.id,
      jobId: application.job_id,
      status: application.status,
      appliedAt: application.applied_at ?? undefined,
      dueDate: application.due_date ?? undefined,
      updatedAt: application.updated_at,
      jobTitle: application.job_title,
      companyName: application.company_name,
    })),
  };
}

// Unsupported legacy endpoints kept only to avoid breaking compile-time imports.
export const createJob = (_payload: JobPostingInput): Promise<JobPosting> =>
  unsupportedPromise('Job creation');
export const fetchJobUrl = (
  _url: string,
): Promise<{ title: string; company: string; description: string }> =>
  unsupportedPromise('Job fetch by URL');
export const uploadResumeFile = (_file: File): Promise<ResumeVersion> =>
  unsupportedPromise('Resume upload');
export const updateJobNote = (_id: string, _note: string): Promise<JobPosting> =>
  unsupported('Job notes');
export const deleteJob = (_id: string): Promise<void> => unsupported('Job deletion');
export const getAlerts = (): Promise<JobAlert[]> => unsupported('Alerts');
export const createAlert = (_payload: JobAlertInput): Promise<JobAlert> =>
  unsupported('Alerts');
export const toggleAlert = (_id: string, _active: boolean): Promise<JobAlert> =>
  unsupported('Alerts');
export const deleteAlert = (_id: string): Promise<void> => unsupported('Alerts');
export const getSuggestedSkills = (): Promise<string[]> => unsupported('Suggested skills');
export const importBatch = (_urls: string[]): Promise<ImportBatchResponse> =>
  unsupported('Batch import');
export const downloadBackup = (): Promise<Record<string, unknown> & BackupMeta> =>
  unsupported('Backup');
export const restoreBackup = (
  _data: unknown,
): Promise<{ restored: boolean; exportedAt: string }> => unsupported('Backup');
