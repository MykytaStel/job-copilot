import type {
  Application,
  ApplicationDetail,
  ApplicationInput,
  ApplicationNote,
  ApplicationOutcome,
  ApplicationStatus,
  Offer,
  OfferInput,
  RejectionStage,
} from '@job-copilot/shared/applications';
import type { DashboardStats } from '@job-copilot/shared/analytics';

import { json, readStoredProfileId, request } from './client';
import type {
  EngineApplication,
  EngineApplicationDetail,
  EngineOffer,
  EngineRecentApplicationsResponse,
} from './engine-types';
import { logUserEvent } from './events';
import { mapApplication, mapApplicationDetail, mapOffer } from './mappers';

export type GlobalSearchApplicationResult = {
  id: string;
  jobId: string;
  status: ApplicationStatus;
  appliedAt?: string;
  dueDate?: string;
  updatedAt: string;
  jobTitle: string;
  companyName: string;
};

export async function getApplications(): Promise<Application[]> {
  const response = await request<EngineRecentApplicationsResponse>('/api/v1/applications/recent');
  return response.applications.map(mapApplication);
}

export async function getApplicationDetail(id: string): Promise<ApplicationDetail> {
  const detail = await request<EngineApplicationDetail>(`/api/v1/applications/${id}`);
  return mapApplicationDetail(detail);
}

export async function createApplication(payload: ApplicationInput): Promise<Application> {
  const application = await request<EngineApplication>(
    '/api/v1/applications',
    json('POST', {
      job_id: payload.jobId,
      status: payload.status,
      applied_at: payload.appliedAt,
    }),
  );

  const profileId = readStoredProfileId();
  if (profileId) {
    void logUserEvent(profileId, {
      eventType: 'application_created',
      jobId: payload.jobId,
      payloadJson: {
        application_id: application.id,
        status: payload.status,
        applied_at: payload.appliedAt ?? null,
      },
    }).catch(() => null);
  }

  return mapApplication(application);
}

export async function updateApplication(
  id: string,
  payload: {
    status?: ApplicationStatus;
    dueDate?: string | null;
    outcome?: ApplicationOutcome | null;
    outcomeDate?: string | null;
    rejectionStage?: RejectionStage | null;
  },
): Promise<Application> {
  const body: Record<string, unknown> = {};

  if (payload.status !== undefined) body.status = payload.status;
  if (payload.dueDate !== undefined) body.due_date = payload.dueDate;
  if (payload.outcome !== undefined) body.outcome = payload.outcome;
  if (payload.outcomeDate !== undefined) body.outcome_date = payload.outcomeDate;
  if (payload.rejectionStage !== undefined) body.rejection_stage = payload.rejectionStage;

  const application = await request<EngineApplication>(
    `/api/v1/applications/${id}`,
    json('PATCH', body),
  );
  return mapApplication(application);
}

export async function patchApplication(
  id: string,
  status: ApplicationStatus,
): Promise<Application> {
  return updateApplication(id, { status });
}

export async function setDueDate(id: string, dueDate: string | null): Promise<Application> {
  return updateApplication(id, { dueDate });
}

export async function addNote(applicationId: string, content: string): Promise<ApplicationNote> {
  const note = await request<{
    id: string;
    content: string;
    created_at: string;
  }>(`/api/v1/applications/${applicationId}/notes`, json('POST', { content }));

  return {
    id: note.id,
    content: note.content,
    createdAt: note.created_at,
  };
}

export async function getDashboardStats(): Promise<DashboardStats> {
  const applications = await getApplications();

  const byStatus: DashboardStats['byStatus'] = {
    saved: 0,
    applied: 0,
    interview: 0,
    offer: 0,
    rejected: 0,
  };

  for (const application of applications) {
    byStatus[application.status] += 1;
  }

  return {
    total: applications.length,
    byStatus,
    topMissingSkills: [],
    avgScore: null,
    tasksDueSoon: 0,
  };
}

export async function createOffer(payload: OfferInput): Promise<Offer> {
  const offer = await request<EngineOffer>(
    `/api/v1/applications/${payload.applicationId}/offer`,
    json('PUT', {
      status: payload.status,
      compensation_min: payload.compensationMin,
      compensation_max: payload.compensationMax,
      compensation_currency: payload.compensationCurrency,
      starts_at: payload.startsAt,
      notes: payload.notes,
    }),
  );

  return mapOffer(offer);
}

