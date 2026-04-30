import type {
  CompanyFeedbackRecord,
  FeedbackOverview,
  JobFeedbackRecord,
  JobFeedbackReason,
  LegitimacySignal,
  SalaryFeedbackSignal,
  WorkModeFeedbackSignal,
} from '@job-copilot/shared/feedback';
import { json, request, requestBlob } from './client';
import type {
  EngineCompanyFeedbackRecord,
  EngineFeedbackOverviewResponse,
  EngineFeedbackStatsResponse,
  EngineFeedbackTimelineResponse,
  EngineJobFeedbackRecord,
} from './engine-types';
import { mapCompanyFeedbackRecord, mapJobFeedbackRecord } from './mappers';
import { invalidateRerankCache } from './reranker';

export type FeedbackStats = {
  savedThisWeekCount: number;
  hiddenThisWeekCount: number;
  badFitThisWeekCount: number;
  whitelistedCompaniesCount: number;
  blacklistedCompaniesCount: number;
};

export type FeedbackExportType = 'saved' | 'hidden' | 'bad_fit' | 'companies';

export type FeedbackTimelineItem = {
  id: string;
  eventType: string;
  jobId?: string | null;
  jobTitle: string;
  companyName: string;
  reason?: string | null;
  createdAt: string;
};

export type FeedbackTimelinePage = {
  profileId: string;
  items: FeedbackTimelineItem[];
  limit: number;
  offset: number;
  totalCount: number;
  nextOffset?: number | null;
};

async function bustRerankCache(profileId: string): Promise<void> {
  await invalidateRerankCache(profileId);
}

export async function getFeedback(profileId: string): Promise<FeedbackOverview> {
  const response = await request<EngineFeedbackOverviewResponse>(
    `/api/v1/profiles/${profileId}/feedback`,
  );

  return {
    profileId: response.profile_id,
    jobs: response.jobs.map(mapJobFeedbackRecord),
    companies: response.companies.map(mapCompanyFeedbackRecord),
    summary: {
      savedJobsCount: response.summary.saved_jobs_count,
      hiddenJobsCount: response.summary.hidden_jobs_count,
      badFitJobsCount: response.summary.bad_fit_jobs_count,
      whitelistedCompaniesCount: response.summary.whitelisted_companies_count,
      blacklistedCompaniesCount: response.summary.blacklisted_companies_count,
    },
  };
}

export async function getFeedbackStats(profileId: string): Promise<FeedbackStats> {
  const response = await request<EngineFeedbackStatsResponse>(
    `/api/v1/feedback/stats?profile_id=${encodeURIComponent(profileId)}`,
  );

  return {
    savedThisWeekCount: response.saved_this_week_count,
    hiddenThisWeekCount: response.hidden_this_week_count,
    badFitThisWeekCount: response.bad_fit_this_week_count,
    whitelistedCompaniesCount: response.whitelisted_companies_count,
    blacklistedCompaniesCount: response.blacklisted_companies_count,
  };
}

export async function getFeedbackTimeline(
  profileId: string,
  options: { limit?: number; offset?: number } = {},
): Promise<FeedbackTimelinePage> {
  const params = new URLSearchParams();
  if (options.limit) params.set('limit', String(options.limit));
  if (options.offset) params.set('offset', String(options.offset));
  const query = params.toString();
  const response = await request<EngineFeedbackTimelineResponse>(
    `/api/v1/profiles/${profileId}/feedback/timeline${query ? `?${query}` : ''}`,
  );

  return {
    profileId: response.profile_id,
    items: response.items.map((item) => ({
      id: item.id,
      eventType: item.event_type,
      jobId: item.job_id,
      jobTitle: item.job_title,
      companyName: item.company_name,
      reason: item.reason,
      createdAt: item.created_at,
    })),
    limit: response.limit,
    offset: response.offset,
    totalCount: response.total_count,
    nextOffset: response.next_offset,
  };
}

export async function exportFeedback(
  exportType: FeedbackExportType,
): Promise<{ blob: Blob; filename?: string }> {
  return requestBlob(`/api/v1/feedback/export?type=${encodeURIComponent(exportType)}`);
}

export async function markJobSaved(profileId: string, jobId: string): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${encodeURIComponent(profileId)}/jobs/${encodeURIComponent(jobId)}/saved`,
    json('PUT', {}),
  );
  await bustRerankCache(profileId);
  return mapJobFeedbackRecord(record);
}

export async function hideJobForProfile(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${encodeURIComponent(profileId)}/jobs/${encodeURIComponent(jobId)}/hidden`,
    json('PUT', {}),
  );
  await bustRerankCache(profileId);
  return mapJobFeedbackRecord(record);
}

export async function bulkHideJobsByCompany(
  profileId: string,
  companyName: string,
): Promise<{ affectedCount: number }> {
  const response = await request<{ affected_count: number }>(
    `/api/v1/feedback/jobs/bulk-hide?profile_id=${encodeURIComponent(profileId)}`,
    json('POST', { company_name: companyName }),
  );
  await bustRerankCache(profileId);
  return { affectedCount: response.affected_count };
}

export async function markJobBadFit(
  profileId: string,
  jobId: string,
  reason?: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/feedback/jobs/${encodeURIComponent(jobId)}/bad-fit?profile_id=${encodeURIComponent(
      profileId,
    )}`,
    json('POST', reason ? { reason } : {}),
  );
  await bustRerankCache(profileId);
  return mapJobFeedbackRecord(record);
}

export async function unsaveJob(profileId: string, jobId: string): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${encodeURIComponent(profileId)}/jobs/${encodeURIComponent(jobId)}/saved`,
    { method: 'DELETE' },
  );
  await bustRerankCache(profileId);
}

export async function unhideJob(profileId: string, jobId: string): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${encodeURIComponent(profileId)}/jobs/${encodeURIComponent(jobId)}/hidden`,
    { method: 'DELETE' },
  );
  await bustRerankCache(profileId);
}

export async function unmarkJobBadFit(profileId: string, jobId: string): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${encodeURIComponent(profileId)}/jobs/${encodeURIComponent(jobId)}/bad-fit`,
    { method: 'DELETE' },
  );
  await bustRerankCache(profileId);
}

export async function undoJobHide(profileId: string, jobId: string): Promise<void> {
  await request<void>(
    `/api/v1/feedback/jobs/${encodeURIComponent(jobId)}/hide?profile_id=${encodeURIComponent(
      profileId,
    )}`,
    { method: 'DELETE' },
  );
  await bustRerankCache(profileId);
}

export async function undoJobBadFit(profileId: string, jobId: string): Promise<void> {
  await request<void>(
    `/api/v1/feedback/jobs/${encodeURIComponent(jobId)}/bad-fit?profile_id=${encodeURIComponent(
      profileId,
    )}`,
    { method: 'DELETE' },
  );
  await bustRerankCache(profileId);
}

export async function addCompanyWhitelist(
  profileId: string,
  companyName: string,
): Promise<CompanyFeedbackRecord> {
  const record = await request<EngineCompanyFeedbackRecord>(
    `/api/v1/profiles/${profileId}/companies/whitelist`,
    json('PUT', { company_name: companyName }),
  );

  return mapCompanyFeedbackRecord(record);
}

export async function removeCompanyWhitelist(
  profileId: string,
  companyName: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/companies/whitelist`,
    json('DELETE', { company_name: companyName }),
  );
}

export async function addCompanyBlacklist(
  profileId: string,
  companyName: string,
): Promise<CompanyFeedbackRecord> {
  const record = await request<EngineCompanyFeedbackRecord>(
    `/api/v1/profiles/${profileId}/companies/blacklist`,
    json('PUT', { company_name: companyName }),
  );

  return mapCompanyFeedbackRecord(record);
}

export async function removeCompanyBlacklist(
  profileId: string,
  companyName: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/companies/blacklist`,
    json('DELETE', { company_name: companyName }),
  );
}

export async function removeCompanyBlacklistBySlug(
  profileId: string,
  companySlug: string,
): Promise<void> {
  const path = `/api/v1/feedback/companies/${encodeURIComponent(
    companySlug,
  )}/blacklist?profile_id=${encodeURIComponent(profileId)}`;

  await request<void>(path, { method: 'DELETE' });
}

export async function updateCompanyFeedbackNotes(
  profileId: string,
  companySlug: string,
  notes: string,
): Promise<CompanyFeedbackRecord> {
  const path = `/api/v1/feedback/companies/${encodeURIComponent(
    companySlug,
  )}?profile_id=${encodeURIComponent(profileId)}`;
  const record = await request<EngineCompanyFeedbackRecord>(path, json('PATCH', { notes }));

  return mapCompanyFeedbackRecord(record);
}

export async function setJobSalarySignal(
  profileId: string,
  jobId: string,
  signal: SalaryFeedbackSignal,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/salary-signal`,
    json('PUT', { signal }),
  );
}

export async function setJobInterestRating(
  profileId: string,
  jobId: string,
  rating: number,
): Promise<void> {
  await request<void>(
    `/api/v1/feedback/jobs/${encodeURIComponent(jobId)}?profile_id=${encodeURIComponent(profileId)}`,
    json('PATCH', { interest_rating: rating }),
  );
}

export async function setJobWorkModeSignal(
  profileId: string,
  jobId: string,
  signal: WorkModeFeedbackSignal,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/work-mode-signal`,
    json('PUT', { signal }),
  );
}

export async function tagJobFeedback(
  profileId: string,
  jobId: string,
  tags: JobFeedbackReason[],
): Promise<void> {
  await request<void>(`/api/v1/profiles/${profileId}/jobs/${jobId}/tags`, json('PUT', { tags }));
}

export async function setJobLegitimacySignal(
  profileId: string,
  jobId: string,
  signal: LegitimacySignal,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/legitimacy-signal`,
    json('PUT', { signal }),
  );
}

export async function clearAllHiddenJobs(profileId: string): Promise<void> {
  await request<void>(`/api/v1/profiles/${profileId}/feedback/hidden/all`, {
    method: 'DELETE',
  });
  await bustRerankCache(profileId);
}
