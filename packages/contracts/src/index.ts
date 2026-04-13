// ─── Job Posting ─────────────────────────────────────────────────────────────

export type JobSource = 'manual' | 'url' | 'rss';

export interface JobPostingInput {
  source: JobSource;
  url?: string;
  rawText?: string;
  title?: string;
  company?: string;
}

export interface JobPosting {
  id: string;
  source: JobSource;
  url?: string;
  title: string;
  company: string;
  description: string;
  notes: string;
  createdAt: string;
  postedAt?: string;
  firstSeenAt?: string;
  lastSeenAt?: string;
  isActive?: boolean;
  inactivatedAt?: string;
  reactivatedAt?: string;
  lifecycleStage?: 'active' | 'inactive' | 'reactivated';
  primaryVariant?: JobSourceVariant;
  salaryMin?: number;
  salaryMax?: number;
  salaryCurrency?: string;
  seniority?: string;
  remoteType?: string;
}

export interface JobSourceVariant {
  source: string;
  sourceJobId: string;
  sourceUrl: string;
  fetchedAt: string;
  lastSeenAt: string;
  isActive: boolean;
  inactivatedAt?: string;
}

export interface JobFeedSummary {
  totalJobs: number;
  activeJobs: number;
  inactiveJobs: number;
  reactivatedJobs: number;
}

// ─── Candidate Profile ────────────────────────────────────────────────────────

export interface CandidateProfile {
  id: string;
  name: string;
  email: string;
  location?: string;
  summary?: string;
  skills: string[];
  updatedAt: string;
  skillsUpdatedAt?: string;
}

export interface CandidateProfileInput {
  name: string;
  email: string;
  location?: string;
  summary?: string;
  skills: string[];
}

// ─── Resume versions ──────────────────────────────────────────────────────────

export interface ResumeVersion {
  id: string;
  version: number;
  filename: string;
  rawText: string;
  isActive: boolean;
  uploadedAt: string;
}

export interface ResumeUploadInput {
  filename: string;
  rawText: string;
}

/** @deprecated use ResumeVersion */
export type Resume = ResumeVersion;

// ─── Match / Fit Score ────────────────────────────────────────────────────────

export interface MatchResult {
  id: string;
  jobId: string;
  resumeId: string;
  /** 0–100 overall fit percentage */
  score: number;
  matchedSkills: string[];
  missingSkills: string[];
  notes: string;
  createdAt: string;
}

// ─── Application ──────────────────────────────────────────────────────────────

export type ApplicationStatus =
  | 'saved'
  | 'applied'
  | 'interview'
  | 'offer'
  | 'rejected';

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
  updatedAt: string;
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

export interface ApplicationInput {
  jobId: string;
  status: ApplicationStatus;
  appliedAt?: string;
}

// ─── Dashboard stats ──────────────────────────────────────────────────────────

export interface DashboardStats {
  total: number;
  byStatus: Record<ApplicationStatus, number>;
  topMissingSkills: Array<{ skill: string; count: number }>;
  avgScore: number | null;
  tasksDueSoon: number;
}

// ─── Contacts / People ────────────────────────────────────────────────────────

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

// ─── Activity log ─────────────────────────────────────────────────────────────

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

// ─── Tasks / Follow-ups ───────────────────────────────────────────────────────

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

// ─── Job Alerts ───────────────────────────────────────────────────────────────

export interface JobAlert {
  id: string;
  /** Keywords to match against job title + description (OR logic) */
  keywords: string[];
  telegramChatId: string;
  active: boolean;
  createdAt: string;
}

export interface JobAlertInput {
  keywords: string[];
  telegramChatId: string;
}

// ─── Cover Letter ─────────────────────────────────────────────────────────────

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

// ─── Interview Q&A ────────────────────────────────────────────────────────────

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

// ─── Offer ────────────────────────────────────────────────────────────────────

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

// ─── Bulk Import ──────────────────────────────────────────────────────────────

export interface ImportResult {
  url: string;
  status: 'imported' | 'duplicate' | 'error';
  job?: JobPosting;
  error?: string;
}

export interface ImportBatchResponse {
  results: ImportResult[];
}

// ─── Backup ───────────────────────────────────────────────────────────────────

export interface BackupMeta {
  version: string;
  exportedAt: string;
}

// ─── Search ───────────────────────────────────────────────────────────────────

export interface SearchResults {
  jobs: JobPosting[];
  contacts: Contact[];
  page: number;
  perPage: number;
  hasMore: boolean;
}

// ─── Health ───────────────────────────────────────────────────────────────────

export interface HealthResponse {
  status: 'ok';
  service: string;
  timestamp: string;
}

// ─── Engine API role + search-profile contracts ─────────────────────────────

export type EngineRoleId =
  | 'react_native_developer'
  | 'mobile_developer'
  | 'frontend_developer'
  | 'backend_developer'
  | 'fullstack_developer'
  | 'qa_engineer'
  | 'devops_engineer'
  | 'data_analyst'
  | 'ui_ux_designer'
  | 'product_manager'
  | 'project_manager'
  | 'marketing_specialist'
  | 'sales_manager'
  | 'customer_support_specialist'
  | 'recruiter'
  | 'generalist';

export type EngineTargetRegion =
  | 'ua'
  | 'eu'
  | 'eu_remote'
  | 'poland'
  | 'germany'
  | 'uk'
  | 'us';

export type EngineWorkMode = 'remote' | 'hybrid' | 'onsite';

export interface EngineRoleCatalogItemResponse {
  id: EngineRoleId;
  display_name: string;
  deprecated_api_ids: string[];
  family?: string;
  is_fallback: boolean;
}

export interface EngineRoleCatalogResponse {
  roles: EngineRoleCatalogItemResponse[];
}

export interface EngineAnalyzeProfileRequest {
  raw_text: string;
}

export interface EngineRoleCandidateResponse {
  role: EngineRoleId;
  score: number;
  confidence: number;
  matched_signals: string[];
}

export interface EngineAnalyzeProfileResponse {
  summary: string;
  primary_role: EngineRoleId;
  seniority: string;
  skills: string[];
  keywords: string[];
  role_candidates: EngineRoleCandidateResponse[];
  suggested_search_terms: string[];
}

export interface EngineSearchPreferencesRequest {
  target_regions?: EngineTargetRegion[];
  work_modes?: EngineWorkMode[];
  preferred_roles?: string[];
  include_keywords?: string[];
  exclude_keywords?: string[];
}

export interface EngineBuildSearchProfileRequest {
  raw_text: string;
  preferences?: EngineSearchPreferencesRequest;
}

export interface EngineSearchProfileResponse {
  primary_role: EngineRoleId;
  target_roles: EngineRoleId[];
  seniority: string;
  target_regions: EngineTargetRegion[];
  work_modes: EngineWorkMode[];
  search_terms: string[];
  exclude_terms: string[];
}

export interface EngineDeprecatedPreferredRoleReplacementResponse {
  received: string;
  normalized_to: EngineRoleId;
}

export interface EngineBuildSearchProfileWarningResponse {
  code: 'deprecated_preferred_roles';
  field: 'preferred_roles';
  message: string;
  replacements: EngineDeprecatedPreferredRoleReplacementResponse[];
}

export interface EngineBuildSearchProfileResponse {
  analyzed_profile: EngineAnalyzeProfileResponse;
  search_profile: EngineSearchProfileResponse;
  warnings?: EngineBuildSearchProfileWarningResponse[];
}

export interface EngineSearchProfileValidationErrorResponse {
  code: 'invalid_preferred_roles';
  field: 'preferred_roles';
  error: 'invalid_preferred_roles';
  message: string;
  invalid_values: string[];
  allowed_values: EngineRoleId[];
}
