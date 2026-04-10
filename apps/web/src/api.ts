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

const API_URL = 'http://localhost:3001';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, init);
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
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

// ─── Health ───────────────────────────────────────────────────────────────────
export const getHealth = () => request<HealthResponse>('/health');

// ─── Jobs ─────────────────────────────────────────────────────────────────────
export const getJobs = () => request<JobPosting[]>('/jobs');
export const getJob = (id: string) => request<JobPosting>(`/jobs/${id}`);
export const createJob = (payload: JobPostingInput) =>
  request<JobPosting>('/jobs/intake', json('POST', payload));
export const fetchJobUrl = (url: string) =>
  request<{ title: string; company: string; description: string }>('/jobs/fetch-url', json('POST', { url }));

// ─── Profile ──────────────────────────────────────────────────────────────────
export const getProfile = () => request<CandidateProfile>('/profile');
export const saveProfile = (payload: CandidateProfileInput) =>
  request<CandidateProfile>('/profile', json('POST', payload));

// ─── Resumes (versioned) ──────────────────────────────────────────────────────
export const getResumes = () => request<ResumeVersion[]>('/resumes');
export const getActiveResume = () => request<ResumeVersion>('/resumes/active');
export const uploadResume = (payload: ResumeUploadInput) =>
  request<ResumeVersion>('/resume/upload', json('POST', payload));
export const uploadResumeFile = (file: File) => {
  const form = new FormData();
  form.append('file', file);
  return request<ResumeVersion>('/resume/upload-file', { method: 'POST', body: form });
};
export const activateResume = (id: string) =>
  request<ResumeVersion>(`/resumes/${id}/activate`, json('POST', {}));

// ─── Match ────────────────────────────────────────────────────────────────────
export const runMatch = (jobId: string) =>
  request<MatchResult>(`/jobs/${jobId}/match`, json('POST', {}));
export const getMatch = (jobId: string) => request<MatchResult>(`/jobs/${jobId}/match`);

// ─── Applications ─────────────────────────────────────────────────────────────
export const getApplications = () => request<Application[]>('/applications');
export const getApplicationDetail = (id: string) =>
  request<ApplicationDetail>(`/applications/${id}`);
export const createApplication = (payload: ApplicationInput) =>
  request<Application>('/applications', json('POST', payload));
export const patchApplication = (id: string, status: ApplicationStatus) =>
  request<Application>(`/applications/${id}`, json('PATCH', { status }));
export const setDueDate = (id: string, dueDate: string | null) =>
  request<Application>(`/applications/${id}`, json('PATCH', { dueDate }));
export const addNote = (applicationId: string, content: string) =>
  request<ApplicationNote>(`/applications/${applicationId}/notes`, json('POST', { content }));

// ─── Job note ─────────────────────────────────────────────────────────────────
export const updateJobNote = (id: string, note: string) =>
  request<JobPosting>(`/jobs/${id}/note`, json('PATCH', { note }));

// ─── Delete ───────────────────────────────────────────────────────────────────
export const deleteJob = (id: string) =>
  request<void>(`/jobs/${id}`, { method: 'DELETE' });
export const deleteApplication = (id: string) =>
  request<void>(`/applications/${id}`, { method: 'DELETE' });

// ─── Market Pulse ─────────────────────────────────────────────────────────────
export const getMarketInsights = () => request<MarketInsights>('/market/insights');

// ─── Alerts ───────────────────────────────────────────────────────────────────
export const getAlerts = () => request<JobAlert[]>('/alerts');
export const createAlert = (payload: JobAlertInput) =>
  request<JobAlert>('/alerts', json('POST', payload));
export const toggleAlert = (id: string, active: boolean) =>
  request<JobAlert>(`/alerts/${id}`, json('PATCH', { active }));
export const deleteAlert = (id: string) =>
  request<void>(`/alerts/${id}`, { method: 'DELETE' });

// ─── Profile suggested skills ─────────────────────────────────────────────────
export const getSuggestedSkills = () => request<string[]>('/profile/suggested-skills');

// ─── Dashboard ────────────────────────────────────────────────────────────────
export const getDashboardStats = () => request<DashboardStats>('/dashboard/stats');

// ─── Search ───────────────────────────────────────────────────────────────────
export const search = (q: string) =>
  request<SearchResults>(`/search?q=${encodeURIComponent(q)}`);

// ─── Contacts ─────────────────────────────────────────────────────────────────
export const getContacts = () => request<Contact[]>('/contacts');
export const createContact = (payload: ContactInput) =>
  request<Contact>('/contacts', json('POST', payload));
export const updateContact = (id: string, payload: Partial<ContactInput>) =>
  request<Contact>(`/contacts/${id}`, json('PATCH', payload));
export const deleteContact = (id: string) =>
  request<void>(`/contacts/${id}`, { method: 'DELETE' });
export const linkContact = (applicationId: string, contactId: string, relationship: string) =>
  request<ApplicationContact>(`/applications/${applicationId}/contacts`, json('POST', { contactId, relationship }));
export const unlinkContact = (applicationId: string, linkId: string) =>
  request<void>(`/applications/${applicationId}/contacts/${linkId}`, { method: 'DELETE' });

// ─── Activities ───────────────────────────────────────────────────────────────
export const getActivities = (applicationId: string) =>
  request<Activity[]>(`/applications/${applicationId}/activities`);
export const createActivity = (applicationId: string, payload: ActivityInput) =>
  request<Activity>(`/applications/${applicationId}/activities`, json('POST', payload));
export const deleteActivity = (id: string) =>
  request<void>(`/activities/${id}`, { method: 'DELETE' });

// ─── Tasks ────────────────────────────────────────────────────────────────────
export const getTasks = (applicationId: string) =>
  request<Task[]>(`/applications/${applicationId}/tasks`);
export const getDueTasks = () => request<Task[]>('/tasks/due');
export const createTask = (applicationId: string, payload: TaskInput) =>
  request<Task>(`/applications/${applicationId}/tasks`, json('POST', payload));
export const patchTask = (id: string, patch: { title?: string; remindAt?: string | null; done?: boolean }) =>
  request<Task>(`/tasks/${id}`, json('PATCH', patch));
export const deleteTask = (id: string) =>
  request<void>(`/tasks/${id}`, { method: 'DELETE' });

// ─── Cover Letters ────────────────────────────────────────────────────────────
export const getCoverLetters = (jobId?: string) =>
  request<CoverLetter[]>(`/cover-letters${jobId ? `?jobId=${jobId}` : ''}`);
export const createCoverLetter = (payload: CoverLetterInput) =>
  request<CoverLetter>('/cover-letters', json('POST', payload));
export const updateCoverLetter = (id: string, content: string) =>
  request<CoverLetter>(`/cover-letters/${id}`, json('PATCH', { content }));
export const deleteCoverLetter = (id: string) =>
  request<void>(`/cover-letters/${id}`, { method: 'DELETE' });

// ─── Interview Q&A ────────────────────────────────────────────────────────────
export const getInterviewQA = (jobId?: string) =>
  request<InterviewQA[]>(`/interview-qa${jobId ? `?jobId=${jobId}` : ''}`);
export const createInterviewQA = (payload: InterviewQAInput) =>
  request<InterviewQA>('/interview-qa', json('POST', payload));
export const updateInterviewQA = (id: string, patch: { question?: string; answer?: string }) =>
  request<InterviewQA>(`/interview-qa/${id}`, json('PATCH', patch));
export const deleteInterviewQA = (id: string) =>
  request<void>(`/interview-qa/${id}`, { method: 'DELETE' });

// ─── Offers ───────────────────────────────────────────────────────────────────
export const getOffers = () => request<Offer[]>('/offers');
export const createOffer = (payload: OfferInput) =>
  request<Offer>('/offers', json('POST', payload));
export const deleteOffer = (id: string) =>
  request<void>(`/offers/${id}`, { method: 'DELETE' });

// ─── Batch Import ─────────────────────────────────────────────────────────────
export const importBatch = (urls: string[]) =>
  request<ImportBatchResponse>('/import/batch', json('POST', { urls }));

// ─── Backup / Restore ─────────────────────────────────────────────────────────
export const downloadBackup = () =>
  request<Record<string, unknown> & BackupMeta>('/backup');
export const restoreBackup = (data: unknown) =>
  request<{ restored: boolean; exportedAt: string }>('/restore', json('POST', data));
