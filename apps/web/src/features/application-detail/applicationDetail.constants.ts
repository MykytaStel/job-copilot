import type {
  ApplicationStatus,
  ContactInput,
  ContactRelationship,
  OfferStatus,
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

export const EMPTY_CONTACT_INPUT: ContactInput = {
  name: '',
  email: '',
  phone: '',
  linkedinUrl: '',
  company: '',
  role: '',
};
