import type {
  ActivityType,
  ApplicationOutcome,
  ApplicationStatus,
  ContactRelationship,
  OfferStatus,
  RejectionStage,
} from '@job-copilot/shared/applications';

import type { EngineContact } from './contacts';
import type { EngineJob } from './jobs';
import type { EngineResume } from './profiles';

export type EngineApplication = {
  id: string;
  job_id: string;
  resume_id: string | null;
  status: ApplicationStatus;
  applied_at: string | null;
  due_date: string | null;
  outcome: ApplicationOutcome | null;
  outcome_date: string | null;
  rejection_stage: RejectionStage | null;
  updated_at: string;
};

export type EngineRecentApplicationsResponse = {
  applications: EngineApplication[];
};

export type EngineGlobalSearchApplication = {
  id: string;
  job_id: string;
  status: ApplicationStatus;
  applied_at: string | null;
  due_date: string | null;
  updated_at: string;
  job_title: string;
  company_name: string;
};

export type EngineOffer = {
  id: string;
  status: OfferStatus;
  compensation_min?: number | null;
  compensation_max?: number | null;
  compensation_currency?: string | null;
  starts_at?: string | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
};

export type EngineApplicationNote = {
  id: string;
  content: string;
  created_at: string;
};

export type EngineApplicationContactLink = {
  id: string;
  relationship: ContactRelationship;
  contact: EngineContact;
};

export type EngineApplicationActivity = {
  id: string;
  activity_type: ActivityType;
  description: string;
  happened_at: string;
  created_at: string;
};

export type EngineApplicationTask = {
  id: string;
  title: string;
  remind_at?: string | null;
  done: boolean;
  created_at: string;
};

export type EngineApplicationDetail = EngineApplication & {
  job: EngineJob;
  resume: EngineResume | null;
  offer?: EngineOffer | null;
  notes: EngineApplicationNote[];
  contacts: EngineApplicationContactLink[];
  activities: EngineApplicationActivity[];
  tasks: EngineApplicationTask[];
};
