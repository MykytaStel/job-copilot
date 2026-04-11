import type {
  Activity,
  ActivityInput,
  Application,
  ApplicationContact,
  ApplicationDetail,
  ApplicationInput,
  ApplicationNote,
  ApplicationStatus,
  BackupMeta,
  CandidateProfile,
  CandidateProfileInput,
  Contact,
  ContactInput,
  CoverLetter,
  CoverLetterInput,
  DashboardStats,
  HealthResponse,
  ImportBatchResponse,
  InterviewQA,
  InterviewQAInput,
  JobAlert,
  JobAlertInput,
  JobPosting,
  JobPostingInput,
  MatchResult,
  Offer,
  OfferInput,
  ResumeUploadInput,
  ResumeVersion,
  SearchResults,
  Task,
  TaskInput,
} from '@job-copilot/shared';

export interface SkillStat {
  skill: string;
  count: number;
  pct: number;
  inResume: boolean;
}

export interface MarketInsights {
  totalJobs: number;
  coverageScore: number;
  topSkills: SkillStat[];
  hotGaps: string[];
  salaryMentions: string[];
}

type EngineApiError = {
  code?: string;
  message?: string;
  details?: unknown;
};

type EngineHealthResponse = {
  status: string;
  database: {
    status: string;
    configured: boolean;
    migrations_enabled_on_startup: boolean;
  };
};

type EngineRecentJobsResponse = {
  jobs: EngineJob[];
};

type EngineRecentApplicationsResponse = {
  applications: EngineApplication[];
};

type EngineResume = {
  id: string;
  version: number;
  filename: string;
  raw_text: string;
  is_active: boolean;
  uploaded_at: string;
};

type EngineMatchResult = {
  id: string;
  job_id: string;
  resume_id: string;
  score: number;
  matched_skills: string[];
  missing_skills: string[];
  notes: string;
  created_at: string;
};

type EngineJob = {
  id: string;
  title: string;
  company_name: string;
  description_text: string;
  posted_at: string | null;
  last_seen_at: string;
};

type EngineApplication = {
  id: string;
  job_id: string;
  resume_id: string | null;
  status: ApplicationStatus;
  applied_at: string | null;
  due_date: string | null;
  updated_at: string;
};

type EngineApplicationDetail = EngineApplication & {
  job: EngineJob;
  resume: EngineResume | null;
  notes: Array<{
    id: string;
    application_id: string;
    content: string;
    created_at: string;
  }>;
  contacts: Array<{
    id: string;
    application_id: string;
    relationship: string;
    contact: {
      id: string;
      name: string;
      email?: string | null;
      phone?: string | null;
      linkedin_url?: string | null;
      company?: string | null;
      role?: string | null;
      created_at: string;
    };
  }>;
  activities: Array<{
    id: string;
    application_id: string;
    activity_type: string;
    description: string;
    happened_at: string;
    created_at: string;
  }>;
  tasks: Array<{
    id: string;
    application_id: string;
    title: string;
    remind_at?: string | null;
    done: boolean;
    created_at: string;
  }>;
};

type EngineProfile = {
  id: string;
  name: string;
  email: string;
  location?: string | null;
  raw_text: string;
  analysis?: {
    summary: string;
    primary_role: string;
    seniority: string;
    skills: string[];
    keywords: string[];
  } | null;
  created_at: string;
  updated_at: string;
};

type EngineAnalyzeProfile = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

const API_URL =
  import.meta.env.VITE_ENGINE_API_URL?.trim() || 'http://localhost:8080';
const PROFILE_ID_KEY = 'engine_api_profile_id';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, init);
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

function json(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  };
}

function unsupported(feature: string): never {
  throw new Error(`${feature} is not supported by engine-api yet`);
}

function unsupportedPromise<T>(feature: string): Promise<T> {
  return Promise.reject(
    new Error(`${feature} is not supported by engine-api yet`),
  );
}

function readStoredProfileId() {
  return window.localStorage.getItem(PROFILE_ID_KEY);
}

function writeStoredProfileId(profileId: string) {
  window.localStorage.setItem(PROFILE_ID_KEY, profileId);
}

function mapJob(job: EngineJob): JobPosting {
  return {
    id: job.id,
    source: 'manual',
    title: job.title,
    company: job.company_name,
    description: job.description_text,
    notes: '',
    createdAt: job.posted_at ?? job.last_seen_at,
  };
}

function mapApplication(application: EngineApplication): Application {
  return {
    id: application.id,
    jobId: application.job_id,
    resumeId: application.resume_id ?? undefined,
    status: application.status,
    appliedAt: application.applied_at ?? undefined,
    dueDate: application.due_date ?? undefined,
    updatedAt: application.updated_at,
  };
}

function mapProfile(profile: EngineProfile): CandidateProfile {
  return {
    id: profile.id,
    name: profile.name,
    email: profile.email,
    location: profile.location ?? undefined,
    summary: profile.analysis?.summary,
    skills: profile.analysis?.skills ?? [],
    updatedAt: profile.updated_at,
  };
}

function mapResume(resume: EngineResume): ResumeVersion {
  return {
    id: resume.id,
    version: resume.version,
    filename: resume.filename,
    rawText: resume.raw_text,
    isActive: resume.is_active,
    uploadedAt: resume.uploaded_at,
  };
}

function mapMatchResult(result: EngineMatchResult): MatchResult {
  return {
    id: result.id,
    jobId: result.job_id,
    resumeId: result.resume_id,
    score: result.score,
    matchedSkills: result.matched_skills,
    missingSkills: result.missing_skills,
    notes: result.notes,
    createdAt: result.created_at,
  };
}

function mapApplicationDetail(detail: EngineApplicationDetail): ApplicationDetail {
  return {
    ...mapApplication(detail),
    job: mapJob(detail.job),
    resume: detail.resume ? mapResume(detail.resume) : undefined,
    notes: detail.notes.map((note) => ({
      id: note.id,
      applicationId: note.application_id,
      content: note.content,
      createdAt: note.created_at,
    })),
    contacts: detail.contacts.map((contact) => ({
      id: contact.id,
      applicationId: contact.application_id,
      relationship: contact.relationship as ApplicationContact['relationship'],
      contact: {
        id: contact.contact.id,
        name: contact.contact.name,
        email: contact.contact.email ?? undefined,
        phone: contact.contact.phone ?? undefined,
        linkedinUrl: contact.contact.linkedin_url ?? undefined,
        company: contact.contact.company ?? undefined,
        role: contact.contact.role ?? undefined,
        createdAt: contact.contact.created_at,
      },
    })),
    activities: detail.activities.map((activity) => ({
      id: activity.id,
      applicationId: activity.application_id,
      type: activity.activity_type as Activity['type'],
      description: activity.description,
      happenedAt: activity.happened_at,
      createdAt: activity.created_at,
    })),
    tasks: detail.tasks.map((task) => ({
      id: task.id,
      applicationId: task.application_id,
      title: task.title,
      remindAt: task.remind_at ?? undefined,
      done: task.done,
      createdAt: task.created_at,
    })),
  };
}

export async function getStoredProfileRawText(): Promise<string> {
  const profileId = readStoredProfileId();
  if (!profileId) return '';

  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return profile.raw_text;
}

export async function analyzeStoredProfile(): Promise<EngineAnalyzeProfile> {
  const profileId = readStoredProfileId();
  if (!profileId) {
    throw new Error('Create a profile first');
  }

  return request<EngineAnalyzeProfile>(
    `/api/v1/profiles/${profileId}/analyze`,
    json('POST', {}),
  );
}

// Supported engine-api endpoints
export async function getHealth(): Promise<HealthResponse> {
  const health = await request<EngineHealthResponse>('/health');

  return {
    status: 'ok',
    service: `engine-api:${health.database.status}`,
    timestamp: new Date().toISOString(),
  };
}

export async function getJobs(): Promise<JobPosting[]> {
  const response = await request<EngineRecentJobsResponse>('/api/v1/jobs/recent');
  return response.jobs.map(mapJob);
}

export async function getJob(id: string): Promise<JobPosting> {
  const job = await request<EngineJob>(`/api/v1/jobs/${id}`);
  return mapJob(job);
}

export async function getApplications(): Promise<Application[]> {
  const response = await request<EngineRecentApplicationsResponse>(
    '/api/v1/applications/recent',
  );
  return response.applications.map(mapApplication);
}

export async function getProfile(): Promise<CandidateProfile | undefined> {
  const profileId = readStoredProfileId();
  if (!profileId) return undefined;

  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return mapProfile(profile);
}

export async function saveProfile(
  payload: CandidateProfileInput & { rawText: string },
): Promise<CandidateProfile> {
  const profileId = readStoredProfileId();

  const body = {
    name: payload.name,
    email: payload.email,
    location: payload.location,
    raw_text: payload.rawText,
  };

  const profile = profileId
    ? await request<EngineProfile>(
        `/api/v1/profiles/${profileId}`,
        json('PATCH', body),
      )
    : await request<EngineProfile>('/api/v1/profiles', json('POST', body));

  writeStoredProfileId(profile.id);

  const analyzed = await request<EngineAnalyzeProfile>(
    `/api/v1/profiles/${profile.id}/analyze`,
    json('POST', {}),
  );

  return {
    id: profile.id,
    name: profile.name,
    email: profile.email,
    location: profile.location ?? undefined,
    summary: analyzed.summary,
    skills: analyzed.skills,
    updatedAt: profile.updated_at,
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

// Unsupported legacy endpoints kept only to avoid breaking compile-time imports.
export const createJob = (_payload: JobPostingInput): Promise<JobPosting> =>
  unsupportedPromise('Job creation');
export const fetchJobUrl = (
  _url: string,
): Promise<{ title: string; company: string; description: string }> =>
  unsupportedPromise('Job fetch by URL');
export async function getResumes(): Promise<ResumeVersion[]> {
  const resumes = await request<EngineResume[]>('/api/v1/resumes');
  return resumes.map(mapResume);
}

export async function getActiveResume(): Promise<ResumeVersion> {
  const resume = await request<EngineResume>('/api/v1/resumes/active');
  return mapResume(resume);
}

export async function uploadResume(
  payload: ResumeUploadInput,
): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    '/api/v1/resume/upload',
    json('POST', {
      filename: payload.filename,
      raw_text: payload.rawText,
    }),
  );
  return mapResume(resume);
}

export const uploadResumeFile = (_file: File): Promise<ResumeVersion> =>
  unsupportedPromise('Resume upload');
export async function activateResume(id: string): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    `/api/v1/resumes/${id}/activate`,
    json('POST', {}),
  );
  return mapResume(resume);
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

export async function getApplicationDetail(id: string): Promise<ApplicationDetail> {
  const detail = await request<EngineApplicationDetail>(`/api/v1/applications/${id}`);
  return mapApplicationDetail(detail);
}

export async function createApplication(
  payload: ApplicationInput,
): Promise<Application> {
  const application = await request<EngineApplication>(
    '/api/v1/applications',
    json('POST', {
      job_id: payload.jobId,
      status: payload.status,
      applied_at: payload.appliedAt,
    }),
  );
  return mapApplication(application);
}

export async function patchApplication(
  id: string,
  status: ApplicationStatus,
): Promise<Application> {
  const application = await request<EngineApplication>(
    `/api/v1/applications/${id}`,
    json('PATCH', { status }),
  );
  return mapApplication(application);
}

export async function setDueDate(
  id: string,
  dueDate: string | null,
): Promise<Application> {
  const application = await request<EngineApplication>(
    `/api/v1/applications/${id}`,
    json('PATCH', { due_date: dueDate }),
  );
  return mapApplication(application);
}
export const addNote = (_applicationId: string, _content: string): Promise<ApplicationNote> =>
  unsupported('Application notes');
export const updateJobNote = (_id: string, _note: string): Promise<JobPosting> =>
  unsupported('Job notes');
export const deleteJob = (_id: string): Promise<void> => unsupported('Job deletion');
export const deleteApplication = (_id: string): Promise<void> =>
  unsupported('Application deletion');
export const getMarketInsights = (): Promise<MarketInsights> =>
  unsupported('Market insights');
export const getAlerts = (): Promise<JobAlert[]> => unsupported('Alerts');
export const createAlert = (_payload: JobAlertInput): Promise<JobAlert> =>
  unsupported('Alerts');
export const toggleAlert = (_id: string, _active: boolean): Promise<JobAlert> =>
  unsupported('Alerts');
export const deleteAlert = (_id: string): Promise<void> => unsupported('Alerts');
export const getSuggestedSkills = (): Promise<string[]> => unsupported('Suggested skills');
export async function search(q: string): Promise<SearchResults> {
  const result = await request<{
    jobs: EngineJob[];
    contacts: Array<{ id: string; name: string; role?: string | null; email?: string | null }>;
  }>(`/api/v1/search?q=${encodeURIComponent(q)}`);

  return {
    jobs: result.jobs.map(mapJob),
    contacts: result.contacts.map((contact) => ({
      id: contact.id,
      name: contact.name,
      role: contact.role ?? undefined,
      email: contact.email ?? undefined,
      createdAt: '',
    })),
  };
}
export const getContacts = (): Promise<Contact[]> => unsupported('Contacts');
export const createContact = (_payload: ContactInput): Promise<Contact> =>
  unsupported('Contacts');
export const updateContact = (
  _id: string,
  _payload: Partial<ContactInput>,
): Promise<Contact> => unsupported('Contacts');
export const deleteContact = (_id: string): Promise<void> => unsupported('Contacts');
export const linkContact = (
  _applicationId: string,
  _contactId: string,
  _relationship: string,
): Promise<ApplicationContact> => unsupported('Application contacts');
export const unlinkContact = (
  _applicationId: string,
  _linkId: string,
): Promise<void> => unsupported('Application contacts');
export const getActivities = (_applicationId: string): Promise<Activity[]> =>
  unsupported('Activities');
export const createActivity = (
  _applicationId: string,
  _payload: ActivityInput,
): Promise<Activity> => unsupported('Activities');
export const deleteActivity = (_id: string): Promise<void> =>
  unsupported('Activities');
export const getTasks = (_applicationId: string): Promise<Task[]> => unsupported('Tasks');
export const getDueTasks = (): Promise<Task[]> => unsupported('Tasks');
export const createTask = (_applicationId: string, _payload: TaskInput): Promise<Task> =>
  unsupported('Tasks');
export const patchTask = (
  _id: string,
  _patch: { title?: string; remindAt?: string | null; done?: boolean },
): Promise<Task> => unsupported('Tasks');
export const deleteTask = (_id: string): Promise<void> => unsupported('Tasks');
export const getCoverLetters = (_jobId?: string): Promise<CoverLetter[]> =>
  unsupported('Cover letters');
export const createCoverLetter = (_payload: CoverLetterInput): Promise<CoverLetter> =>
  unsupported('Cover letters');
export const updateCoverLetter = (_id: string, _content: string): Promise<CoverLetter> =>
  unsupported('Cover letters');
export const deleteCoverLetter = (_id: string): Promise<void> =>
  unsupported('Cover letters');
export const getInterviewQA = (_jobId?: string): Promise<InterviewQA[]> =>
  unsupported('Interview Q&A');
export const createInterviewQA = (_payload: InterviewQAInput): Promise<InterviewQA> =>
  unsupported('Interview Q&A');
export const updateInterviewQA = (
  _id: string,
  _patch: { question?: string; answer?: string },
): Promise<InterviewQA> => unsupported('Interview Q&A');
export const deleteInterviewQA = (_id: string): Promise<void> =>
  unsupported('Interview Q&A');
export const getOffers = (): Promise<Offer[]> => unsupported('Offers');
export const createOffer = (_payload: OfferInput): Promise<Offer> => unsupported('Offers');
export const deleteOffer = (_id: string): Promise<void> => unsupported('Offers');
export const importBatch = (_urls: string[]): Promise<ImportBatchResponse> =>
  unsupported('Batch import');
export const downloadBackup = (): Promise<Record<string, unknown> & BackupMeta> =>
  unsupported('Backup');
export const restoreBackup = (
  _data: unknown,
): Promise<{ restored: boolean; exportedAt: string }> => unsupported('Backup');
