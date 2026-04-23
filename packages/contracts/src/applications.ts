import type { JobPosting } from './jobs';
import type { ResumeVersion } from './profiles';

export type ApplicationStatus =
  | 'saved'
  | 'applied'
  | 'interview'
  | 'offer'
  | 'rejected';

export type ApplicationOutcome =
  | 'phone_screen'
  | 'technical_interview'
  | 'final_interview'
  | 'offer_received'
  | 'rejected'
  | 'ghosted'
  | 'withdrew';

export type RejectionStage =
  | 'applied'
  | 'phone_screen'
  | 'technical_interview'
  | 'final_interview';

export interface ApplicationNote {
  id: string;
  applicationId: string;
  content: string;
  createdAt: string;
}

export interface Application {
  id: string;
  jobId: string;
  /** Which resume version was sent with this application */
  resumeId?: string;
  status: ApplicationStatus;
  appliedAt?: string;
  dueDate?: string;
  outcome?: ApplicationOutcome;
  outcomeDate?: string;
  rejectionStage?: RejectionStage;
  updatedAt: string;
}

export interface ApplicationInput {
  jobId: string;
  status: ApplicationStatus;
  appliedAt?: string;
}

export type ContactRelationship =
  | 'recruiter'
  | 'hiring_manager'
  | 'interviewer'
  | 'referrer'
  | 'other';

export interface Contact {
  id: string;
  name: string;
  email?: string;
  phone?: string;
  linkedinUrl?: string;
  company?: string;
  role?: string;
  createdAt: string;
}

export interface ContactInput {
  name: string;
  email?: string;
  phone?: string;
  linkedinUrl?: string;
  company?: string;
  role?: string;
}

export interface ApplicationContact {
  id: string;
  applicationId: string;
  contact: Contact;
  relationship: ContactRelationship;
}

export type ActivityType = 'email' | 'call' | 'interview' | 'follow_up' | 'note' | 'other';

export interface Activity {
  id: string;
  applicationId: string;
  type: ActivityType;
  description: string;
  happenedAt: string;
  createdAt: string;
}

export interface ActivityInput {
  type: ActivityType;
  description: string;
  happenedAt: string;
}

export interface Task {
  id: string;
  applicationId: string;
  title: string;
  remindAt?: string;
  done: boolean;
  createdAt: string;
}

export interface TaskInput {
  title: string;
  remindAt?: string;
}

export type CoverLetterTone = 'formal' | 'casual' | 'enthusiastic';

export interface CoverLetter {
  id: string;
  jobId: string;
  content: string;
  tone: CoverLetterTone;
  createdAt: string;
}

export interface CoverLetterInput {
  jobId: string;
  tone: CoverLetterTone;
  /** If provided, saves this content directly (no AI call) */
  content?: string;
}

export type InterviewCategory = 'behavioral' | 'technical' | 'situational' | 'company';

export interface InterviewQA {
  id: string;
  jobId: string;
  question: string;
  answer: string;
  category: InterviewCategory;
  createdAt: string;
}

export interface InterviewQAInput {
  jobId: string;
  question: string;
  answer?: string;
  category: InterviewCategory;
}

export type OfferStatus = 'draft' | 'received' | 'accepted' | 'declined' | 'expired';

export interface Offer {
  id: string;
  applicationId: string;
  status: OfferStatus;
  compensationMin?: number;
  compensationMax?: number;
  compensationCurrency?: string;
  startsAt?: string;
  notes?: string;
  createdAt: string;
  updatedAt: string;
}

export interface OfferInput {
  applicationId: string;
  status: OfferStatus;
  compensationMin?: number;
  compensationMax?: number;
  compensationCurrency?: string;
  startsAt?: string;
  notes?: string;
}

export interface ApplicationDetail extends Application {
  job: JobPosting;
  resume?: ResumeVersion;
  offer?: Offer;
  notes: ApplicationNote[];
  contacts: ApplicationContact[];
  activities: Activity[];
  tasks: Task[];
}
