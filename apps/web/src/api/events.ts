import { json, request } from './client';

export type UserEventType =
  | 'job_impression'
  | 'job_opened'
  | 'job_saved'
  | 'job_unsaved'
  | 'job_hidden'
  | 'job_unhidden'
  | 'job_bad_fit'
  | 'job_bad_fit_removed'
  | 'company_whitelisted'
  | 'company_blacklisted'
  | 'search_run'
  | 'fit_explanation_requested'
  | 'application_coach_requested'
  | 'cover_letter_draft_requested'
  | 'interview_prep_requested'
  | 'application_created'
  | 'job_scrolled_to_bottom'
  | 'job_returned'
  | 'job_shared';

type UserEventLogInput = {
  eventType: UserEventType;
  jobId?: string;
  companyName?: string;
  source?: string;
  roleFamily?: string;
  payloadJson?: Record<string, unknown>;
};

export async function logUserEvent(profileId: string, input: UserEventLogInput): Promise<void> {
  await request<void>(
    `/api/v1/profiles/${profileId}/events`,
    json('POST', {
      event_type: input.eventType,
      job_id: input.jobId,
      company_name: input.companyName,
      source: input.source,
      role_family: input.roleFamily,
      payload_json: input.payloadJson,
    }),
  );
}
