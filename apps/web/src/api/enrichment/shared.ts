import type { JobFeedbackState } from '@job-copilot/shared/feedback';
import type { JobPosting } from '@job-copilot/shared/jobs';

import type {
  AnalyticsFeedbackSummary,
  AnalyticsSummary,
  BehaviorSummary,
  FunnelSummary,
  LlmContext,
  LlmContextAnalyzedProfile,
  LlmContextEvidenceEntry,
} from '../analytics';
import type { FitExplanation } from '../jobs';
import type { SearchProfileBuildResult } from '../profiles';

import type {
  ApplicationCoach,
  CoverLetterDraft,
  JobFitExplanation,
} from './types';

export type AnalyzedProfileInput = LlmContextAnalyzedProfile & {
  roleCandidates?: Array<{ role: string; confidence: number }>;
  suggestedSearchTerms?: string[];
};

export type SearchProfileInput = {
  primaryRole: string;
  primaryRoleConfidence?: number;
  targetRoles: string[];
  roleCandidates: Array<{ role: string; confidence: number }>;
  seniority: string;
  targetRegions: string[];
  workModes: string[];
  allowedSources: string[];
  profileSkills: string[];
  profileKeywords: string[];
  searchTerms: string[];
  excludeTerms: string[];
};

export type DeterministicFitInput = {
  jobId: string;
  score: number;
  matchedRoles: string[];
  matchedSkills: string[];
  matchedKeywords: string[];
  sourceMatch: boolean;
  workModeMatch?: boolean;
  regionMatch?: boolean;
  reasons: string[];
};

export type JobFitExplanationInput = {
  fitSummary: string;
  whyItMatches: string[];
  risks: string[];
  missingSignals: string[];
  recommendedNextStep: string;
  applicationAngle: string;
};

export type ApplicationCoachInput = {
  applicationSummary: string;
  resumeFocusPoints: string[];
  suggestedBullets: string[];
  coverLetterAngles: string[];
  interviewFocus: string[];
  gapsToAddress: string[];
  redFlags: string[];
};

export type CoverLetterDraftInput = {
  draftSummary: string;
  openingParagraph: string;
  bodyParagraphs: string[];
  closingParagraph: string;
  keyClaimsUsed: string[];
  evidenceGaps: string[];
  toneNotes: string[];
};

export type FeedbackStateInput = {
  feedbackSummary: AnalyticsFeedbackSummary;
  topPositiveEvidence: LlmContextEvidenceEntry[];
  topNegativeEvidence: LlmContextEvidenceEntry[];
  currentJobFeedback?: JobFeedbackState;
};

export type WeeklyGuidanceRequest = {
  profileId: string;
  analyticsSummary: AnalyticsSummary;
  behaviorSummary: BehaviorSummary;
  funnelSummary: FunnelSummary;
  llmContext: LlmContext;
};

type EnrichmentFeedbackState = {
  feedbackSummary: AnalyticsFeedbackSummary;
  topPositiveEvidence: LlmContextEvidenceEntry[];
  topNegativeEvidence: LlmContextEvidenceEntry[];
  currentJobFeedback?: JobFeedbackState;
};

export type JobFitExplanationRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  feedbackState?: EnrichmentFeedbackState | null;
};

export type ApplicationCoachRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  feedbackState?: EnrichmentFeedbackState | null;
  rawProfileText?: string | null;
};

export type CoverLetterDraftRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  applicationCoach?: ApplicationCoach | null;
  feedbackState?: EnrichmentFeedbackState | null;
  rawProfileText?: string | null;
};

export type InterviewPrepRequest = {
  profileId: string;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'] | LlmContextAnalyzedProfile | null;
  searchProfile: SearchProfileBuildResult['searchProfile'] | null;
  rankedJob: JobPosting;
  deterministicFit: FitExplanation;
  jobFitExplanation?: JobFitExplanation | null;
  applicationCoach?: ApplicationCoach | null;
  coverLetterDraft?: CoverLetterDraft | null;
  feedbackState?: EnrichmentFeedbackState | null;
  rawProfileText?: string | null;
};

export function buildAnalyzedProfilePayload(profile: AnalyzedProfileInput | null) {
  if (!profile) {
    return null;
  }

  return {
    summary: profile.summary,
    primary_role: profile.primaryRole,
    seniority: profile.seniority,
    skills: profile.skills,
    keywords: profile.keywords,
  };
}

export function buildSearchProfilePayload(profile: SearchProfileInput | null) {
  if (!profile) {
    return null;
  }

  return {
    primary_role: profile.primaryRole,
    primary_role_confidence: profile.primaryRoleConfidence,
    target_roles: profile.targetRoles,
    role_candidates: profile.roleCandidates,
    seniority: profile.seniority,
    target_regions: profile.targetRegions,
    work_modes: profile.workModes,
    allowed_sources: profile.allowedSources,
    profile_skills: profile.profileSkills,
    profile_keywords: profile.profileKeywords,
    search_terms: profile.searchTerms,
    exclude_terms: profile.excludeTerms,
  };
}

export function buildRankedJobPayload(job: JobPosting) {
  return {
    id: job.id,
    title: job.title,
    company_name: job.company,
    description_text: job.description,
    summary: job.presentation?.summary,
    source: job.primaryVariant?.source,
    source_url: job.primaryVariant?.sourceUrl ?? job.url,
    remote_type: job.remoteType,
    seniority: job.seniority,
    salary_label: job.presentation?.salaryLabel,
    location_label: job.presentation?.locationLabel,
    work_mode_label: job.presentation?.workModeLabel,
    freshness_label: job.presentation?.freshnessLabel,
    badges: job.presentation?.badges ?? [],
  };
}

export function buildDeterministicFitPayload(fit: DeterministicFitInput) {
  return {
    job_id: fit.jobId,
    score: fit.score,
    matched_roles: fit.matchedRoles,
    matched_skills: fit.matchedSkills,
    matched_keywords: fit.matchedKeywords,
    source_match: fit.sourceMatch,
    work_mode_match: fit.workModeMatch,
    region_match: fit.regionMatch,
    reasons: fit.reasons,
  };
}

export function buildFeedbackSummaryPayload(summary: AnalyticsFeedbackSummary) {
  return {
    saved_jobs_count: summary.savedJobsCount,
    hidden_jobs_count: summary.hiddenJobsCount,
    bad_fit_jobs_count: summary.badFitJobsCount,
    whitelisted_companies_count: summary.whitelistedCompaniesCount,
    blacklisted_companies_count: summary.blacklistedCompaniesCount,
  };
}

export function buildEvidencePayload(entries: LlmContextEvidenceEntry[]) {
  return entries.map((entry) => ({
    type: entry.type,
    label: entry.label,
  }));
}

export function buildFeedbackStatePayload(feedbackState?: FeedbackStateInput | null) {
  if (!feedbackState) {
    return null;
  }

  return {
    summary: buildFeedbackSummaryPayload(feedbackState.feedbackSummary),
    top_positive_evidence: buildEvidencePayload(feedbackState.topPositiveEvidence),
    top_negative_evidence: buildEvidencePayload(feedbackState.topNegativeEvidence),
    current_job_feedback: feedbackState.currentJobFeedback
      ? {
          saved: feedbackState.currentJobFeedback.saved,
          hidden: feedbackState.currentJobFeedback.hidden,
          bad_fit: feedbackState.currentJobFeedback.badFit,
          company_status: feedbackState.currentJobFeedback.companyStatus,
        }
      : null,
  };
}

export function buildLlmContextPayload(
  context: LlmContext,
  options?: { includeProfileId?: boolean },
) {
  const payload = {
    analyzed_profile: buildAnalyzedProfilePayload(context.analyzedProfile),
    profile_skills: context.profileSkills,
    profile_keywords: context.profileKeywords,
    jobs_feed_summary: {
      total: context.jobsFeedSummary.total,
      active: context.jobsFeedSummary.active,
      inactive: context.jobsFeedSummary.inactive,
      reactivated: context.jobsFeedSummary.reactivated,
    },
    feedback_summary: buildFeedbackSummaryPayload(context.feedbackSummary),
    top_positive_evidence: buildEvidencePayload(context.topPositiveEvidence),
    top_negative_evidence: buildEvidencePayload(context.topNegativeEvidence),
  };

  if (options?.includeProfileId === false) {
    return payload;
  }

  return {
    profile_id: context.profileId,
    ...payload,
  };
}

export function buildJobFitExplanationSummary(jobFitExplanation?: JobFitExplanationInput | null) {
  if (!jobFitExplanation) {
    return null;
  }

  return {
    fit_summary: jobFitExplanation.fitSummary,
    why_it_matches: jobFitExplanation.whyItMatches,
    risks: jobFitExplanation.risks,
    missing_signals: jobFitExplanation.missingSignals,
    recommended_next_step: jobFitExplanation.recommendedNextStep,
    application_angle: jobFitExplanation.applicationAngle,
  };
}

export function buildApplicationCoachSummary(applicationCoach?: ApplicationCoachInput | null) {
  if (!applicationCoach) {
    return null;
  }

  return {
    application_summary: applicationCoach.applicationSummary,
    resume_focus_points: applicationCoach.resumeFocusPoints,
    suggested_bullets: applicationCoach.suggestedBullets,
    cover_letter_angles: applicationCoach.coverLetterAngles,
    interview_focus: applicationCoach.interviewFocus,
    gaps_to_address: applicationCoach.gapsToAddress,
    red_flags: applicationCoach.redFlags,
  };
}

export function buildCoverLetterDraftSummary(coverLetterDraft?: CoverLetterDraftInput | null) {
  if (!coverLetterDraft) {
    return null;
  }

  return {
    draft_summary: coverLetterDraft.draftSummary,
    opening_paragraph: coverLetterDraft.openingParagraph,
    body_paragraphs: coverLetterDraft.bodyParagraphs,
    closing_paragraph: coverLetterDraft.closingParagraph,
    key_claims_used: coverLetterDraft.keyClaimsUsed,
    evidence_gaps: coverLetterDraft.evidenceGaps,
    tone_notes: coverLetterDraft.toneNotes,
  };
}
