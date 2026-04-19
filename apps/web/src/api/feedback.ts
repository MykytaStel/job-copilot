import type {
  CompanyFeedbackRecord,
  FeedbackOverview,
  JobFeedbackRecord,
} from '@job-copilot/shared';
import { json, request } from './client';
import type {
  EngineCompanyFeedbackRecord,
  EngineFeedbackOverviewResponse,
  EngineJobFeedbackRecord,
} from './engine-types';
import { mapCompanyFeedbackRecord, mapJobFeedbackRecord } from './mappers';

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

export async function markJobSaved(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/saved`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function hideJobForProfile(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/hidden`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function markJobBadFit(
  profileId: string,
  jobId: string,
): Promise<JobFeedbackRecord> {
  const record = await request<EngineJobFeedbackRecord>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/bad-fit`,
    json('PUT', {}),
  );

  return mapJobFeedbackRecord(record);
}

export async function unsaveJob(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/saved`,
    { method: 'DELETE' },
  );
}

export async function unhideJob(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/hidden`,
    { method: 'DELETE' },
  );
}

export async function unmarkJobBadFit(
  profileId: string,
  jobId: string,
): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/jobs/${jobId}/bad-fit`,
    { method: 'DELETE' },
  );
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
