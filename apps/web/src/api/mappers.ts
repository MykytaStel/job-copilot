import type {
  Application,
  ApplicationDetail,
  Contact,
  Offer,
} from '@job-copilot/shared/applications';
import type { CandidateProfile, ResumeVersion } from '@job-copilot/shared/profiles';
import type {
  CompanyFeedbackRecord,
  JobFeedbackRecord,
  JobFeedbackState,
} from '@job-copilot/shared/feedback';
import type {
  JobFeedSummary,
  JobPosting,
  MatchResult,
} from '@job-copilot/shared/jobs';

import type {
  EngineApplication,
  EngineApplicationDetail,
  EngineCompanyFeedbackRecord,
  EngineContact,
  EngineJob,
  EngineJobFeedbackRecord,
  EngineJobFeedbackState,
  EngineJobFeedSummary,
  EngineMatchResult,
  EngineOffer,
  EngineProfile,
  EngineResume,
} from './engine-types';

export function uniquePreservingOrder(values: string[]): string[] {
  const seen = new Set<string>();
  const result: string[] = [];

  for (const value of values) {
    const normalized = value.trim();
    if (!normalized || seen.has(normalized)) {
      continue;
    }
    seen.add(normalized);
    result.push(normalized);
  }

  return result;
}

export function normalizeMissingString(value?: string | null): string | undefined {
  if (!value) return undefined;
  const cleaned = value.trim();
  if (!cleaned || cleaned.toLowerCase() === 'unknown') {
    return undefined;
  }
  return cleaned;
}

function mapJobFeedbackState(feedback: EngineJobFeedbackState): JobFeedbackState {
  return {
    saved: feedback.saved,
    hidden: feedback.hidden,
    badFit: feedback.bad_fit,
    companyStatus: feedback.company_status ?? undefined,
    salarySignal: feedback.salary_signal ?? undefined,
    interestRating: feedback.interest_rating ?? undefined,
    workModeSignal: feedback.work_mode_signal ?? undefined,
    legitimacySignal: feedback.legitimacy_signal ?? undefined,
    tags: feedback.tags ?? undefined,
  };
}

export function mapJob(job: EngineJob): JobPosting {
  return {
    id: job.id,
    source: 'manual',
    url: job.presentation.outbound_url ?? job.primary_variant?.source_url ?? undefined,
    title: job.presentation.title || job.title,
    company: job.presentation.company || job.company_name,
    description: job.description_text,
    location: normalizeMissingString(job.location),
    notes: '',
    createdAt: job.posted_at ?? job.last_seen_at,
    postedAt: job.posted_at ?? undefined,
    firstSeenAt: job.first_seen_at,
    lastSeenAt: job.last_seen_at,
    isActive: job.is_active,
    inactivatedAt: job.inactivated_at ?? undefined,
    reactivatedAt: job.reactivated_at ?? undefined,
    lifecycleStage: job.lifecycle_stage,
    salaryMin: job.salary_min ?? undefined,
    salaryMax: job.salary_max ?? undefined,
    salaryCurrency: job.salary_currency ?? undefined,
    seniority: normalizeMissingString(job.seniority),
    remoteType: job.remote_type ?? undefined,
    primaryVariant: job.primary_variant
      ? {
          source: job.primary_variant.source,
          sourceUrl: job.primary_variant.source_url,
          fetchedAt: job.primary_variant.fetched_at,
          lastSeenAt: job.primary_variant.last_seen_at,
          isActive: job.primary_variant.is_active,
          inactivatedAt: job.primary_variant.inactivated_at ?? undefined,
        }
      : undefined,
    presentation: {
      title: job.presentation.title,
      company: job.presentation.company,
      summary: job.presentation.summary ?? undefined,
      summaryQuality: job.presentation.summary_quality ?? undefined,
      summaryFallback: job.presentation.summary_fallback,
      descriptionQuality: job.presentation.description_quality,
      locationLabel: job.presentation.location_label ?? undefined,
      workModeLabel: job.presentation.work_mode_label ?? undefined,
      sourceLabel: job.presentation.source_label ?? undefined,
      outboundUrl: job.presentation.outbound_url ?? undefined,
      salaryLabel: job.presentation.salary_label ?? undefined,
      freshnessLabel: job.presentation.freshness_label ?? undefined,
      lifecyclePrimaryLabel: job.presentation.lifecycle_primary_label ?? undefined,
      lifecycleSecondaryLabel: job.presentation.lifecycle_secondary_label ?? undefined,
      badges: job.presentation.badges,
    },
    feedback: mapJobFeedbackState(job.feedback),
  };
}

export function mapJobFeedbackRecord(record: EngineJobFeedbackRecord): JobFeedbackRecord {
  return {
    jobId: record.job_id,
    saved: record.saved,
    hidden: record.hidden,
    badFit: record.bad_fit,
    updatedAt: record.updated_at,
  };
}

export function mapCompanyFeedbackRecord(
  record: EngineCompanyFeedbackRecord,
): CompanyFeedbackRecord {
  return {
    companyName: record.company_name,
    normalizedCompanyName: record.normalized_company_name,
    status: record.status,
    updatedAt: record.updated_at,
  };
}

export function mapJobFeedSummary(summary: EngineJobFeedSummary): JobFeedSummary {
  return {
    totalJobs: summary.total_jobs,
    activeJobs: summary.active_jobs,
    inactiveJobs: summary.inactive_jobs,
    reactivatedJobs: summary.reactivated_jobs,
  };
}

export function mapApplication(application: EngineApplication): Application {
  return {
    id: application.id,
    jobId: application.job_id,
    resumeId: application.resume_id ?? undefined,
    status: application.status,
    appliedAt: application.applied_at ?? undefined,
    dueDate: application.due_date ?? undefined,
    outcome: application.outcome ?? undefined,
    outcomeDate: application.outcome_date ?? undefined,
    rejectionStage: application.rejection_stage ?? undefined,
    updatedAt: application.updated_at,
  };
}

export function mapProfile(profile: EngineProfile): CandidateProfile {
  return {
    id: profile.id,
    name: profile.name,
    email: profile.email,
    location: profile.location ?? undefined,
    yearsOfExperience: profile.years_of_experience ?? undefined,
    salaryMin: profile.salary_min ?? undefined,
    salaryMax: profile.salary_max ?? undefined,
    salaryCurrency: profile.salary_currency ?? 'USD',
    languages: profile.languages ?? [],
    preferredLocations: profile.preferred_locations ?? [],
    workModePreference: profile.work_mode_preference ?? 'any',
    summary: profile.analysis?.summary,
    skills: profile.analysis?.skills ?? [],
    updatedAt: profile.updated_at,
    skillsUpdatedAt: profile.skills_updated_at ?? undefined,
		portfolioUrl: profile.portfolio_url ?? undefined,
		githubUrl: profile.github_url ?? undefined,
		linkedinUrl: profile.linkedin_url ?? undefined,
  };
}

export function mapContact(contact: EngineContact): Contact {
  return {
    id: contact.id,
    name: contact.name,
    email: contact.email ?? undefined,
    phone: contact.phone ?? undefined,
    linkedinUrl: contact.linkedin_url ?? undefined,
    company: contact.company ?? undefined,
    role: contact.role ?? undefined,
    createdAt: contact.created_at,
  };
}

export function mapOffer(offer: EngineOffer): Offer {
  return {
    id: offer.id,
    status: offer.status,
    compensationMin: offer.compensation_min ?? undefined,
    compensationMax: offer.compensation_max ?? undefined,
    compensationCurrency: offer.compensation_currency ?? undefined,
    startsAt: offer.starts_at ?? undefined,
    notes: offer.notes ?? undefined,
    createdAt: offer.created_at,
    updatedAt: offer.updated_at,
  };
}

export function mapResume(resume: EngineResume): ResumeVersion {
  return {
    id: resume.id,
    version: resume.version,
    filename: resume.filename,
    rawText: resume.raw_text,
    isActive: resume.is_active,
    uploadedAt: resume.uploaded_at,
  };
}

export function mapMatchResult(result: EngineMatchResult): MatchResult {
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

export function mapApplicationDetail(detail: EngineApplicationDetail): ApplicationDetail {
  return {
    ...mapApplication(detail),
    job: mapJob(detail.job),
    resume: detail.resume ? mapResume(detail.resume) : undefined,
    offer: detail.offer ? mapOffer(detail.offer) : undefined,
    notes: detail.notes.map((note) => ({
      id: note.id,
      content: note.content,
      createdAt: note.created_at,
    })),
    contacts: detail.contacts.map((contact) => ({
      id: contact.id,
      relationship: contact.relationship,
      contact: mapContact(contact.contact),
    })),
    activities: detail.activities.map((activity) => ({
      id: activity.id,
      type: activity.activity_type,
      description: activity.description,
      happenedAt: activity.happened_at,
      createdAt: activity.created_at,
    })),
    tasks: detail.tasks.map((task) => ({
      id: task.id,
      title: task.title,
      remindAt: task.remind_at ?? undefined,
      done: task.done,
      createdAt: task.created_at,
    })),
  };
}
