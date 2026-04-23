import type {
  ApplicationOutcome,
  ApplicationStatus,
  ContactInput,
  ContactRelationship,
  OfferStatus,
  RejectionStage,
} from '@job-copilot/shared';

export const RELATIONSHIP_OPTIONS: ContactRelationship[] = [
  'recruiter',
  'hiring_manager',
  'interviewer',
  'referrer',
  'other',
];

export const OFFER_STATUS_OPTIONS: OfferStatus[] = [
  'draft',
  'received',
  'accepted',
  'declined',
  'expired',
];

export const APPLICATION_STATUS_OPTIONS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

export const APPLICATION_OUTCOME_OPTIONS: Array<ApplicationOutcome | ''> = [
  '',
  'phone_screen',
  'technical_interview',
  'final_interview',
  'offer_received',
  'rejected',
  'ghosted',
  'withdrew',
];

export const REJECTION_STAGE_OPTIONS: Array<RejectionStage | ''> = [
  '',
  'applied',
  'phone_screen',
  'technical_interview',
  'final_interview',
];

export const EMPTY_CONTACT_INPUT: ContactInput = {
  name: '',
  email: '',
  phone: '',
  linkedinUrl: '',
  company: '',
  role: '',
};
