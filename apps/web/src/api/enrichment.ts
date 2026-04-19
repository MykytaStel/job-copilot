import type { JobFeedbackState, JobPosting } from '@job-copilot/shared';

import type {
  AnalyticsFeedbackSummary,
  AnalyticsSummary,
  BehaviorSummary,
  FunnelSummary,
  LlmContext,
  LlmContextAnalyzedProfile,
  LlmContextEvidenceEntry,
} from './analytics';

type AnalyzedProfileInput = LlmContextAnalyzedProfile & {
  roleCandidates?: Array<{ role: string; confidence: number }>;
  suggestedSearchTerms?: string[];
};

type SearchProfileInput = {
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

type DeterministicFitInput = {
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

type JobFitExplanationInput = {
  fitSummary: string;
  whyItMatches: string[];
  risks: string[];
  missingSignals: string[];
  recommendedNextStep: string;
  applicationAngle: string;
};

type ApplicationCoachInput = {
  applicationSummary: string;
  resumeFocusPoints: string[];
  suggestedBullets: string[];
  coverLetterAngles: string[];
  interviewFocus: string[];
  gapsToAddress: string[];
  redFlags: string[];
};

type CoverLetterDraftInput = {
  draftSummary: string;
  openingParagraph: string;
  bodyParagraphs: string[];
  closingParagraph: string;
  keyClaimsUsed: string[];
  evidenceGaps: string[];
  toneNotes: string[];
};

type FeedbackStateInput = {
  feedbackSummary: AnalyticsFeedbackSummary;
  topPositiveEvidence: LlmContextEvidenceEntry[];
  topNegativeEvidence: LlmContextEvidenceEntry[];
  currentJobFeedback?: JobFeedbackState;
};

export type MlProfileInsightsResponse = {
  profile_summary: string;
  search_strategy_summary: string;
  strengths: string[];
  risks: string[];
  recommended_actions: string[];
  top_focus_areas: string[];
  search_term_suggestions: string[];
  application_strategy: string[];
};

export type MlJobFitExplanationResponse = {
  fit_summary: string;
  why_it_matches: string[];
  risks: string[];
  missing_signals: string[];
  recommended_next_step: string;
  application_angle: string;
};

export type MlApplicationCoachResponse = {
  application_summary: string;
  resume_focus_points: string[];
  suggested_bullets: string[];
  cover_letter_angles: string[];
  interview_focus: string[];
  gaps_to_address: string[];
  red_flags: string[];
};

export type MlCoverLetterDraftResponse = {
  draft_summary: string;
  opening_paragraph: string;
  body_paragraphs: string[];
  closing_paragraph: string;
  key_claims_used: string[];
  evidence_gaps: string[];
  tone_notes: string[];
};

export type MlInterviewPrepResponse = {
  prep_summary: string;
  likely_topics: string[];
  technical_focus: string[];
  behavioral_focus: string[];
  stories_to_prepare: string[];
  questions_to_ask: string[];
  risk_areas: string[];
  follow_up_plan: string[];
};

export type MlWeeklyGuidanceResponse = {
  weekly_summary: string;
  what_is_working: string[];
  what_is_not_working: string[];
  recommended_search_adjustments: string[];
  recommended_source_moves: string[];
  recommended_role_focus: string[];
  funnel_bottlenecks: string[];
  next_week_plan: string[];
};

function buildAnalyzedProfilePayload(profile: AnalyzedProfileInput | null) {
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

function buildSearchProfilePayload(profile: SearchProfileInput | null) {
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

function buildRankedJobPayload(job: JobPosting) {
  return {
    id: job.id,
    title: job.title,
    company_name: job.company,
    description_text: job.description,
    summary: job.presentation?.summary,
    source: job.primaryVariant?.source,
    source_job_id: job.primaryVariant?.sourceJobId,
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

function buildDeterministicFitPayload(fit: DeterministicFitInput) {
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

function buildFeedbackSummaryPayload(summary: AnalyticsFeedbackSummary) {
  return {
    saved_jobs_count: summary.savedJobsCount,
    hidden_jobs_count: summary.hiddenJobsCount,
    bad_fit_jobs_count: summary.badFitJobsCount,
    whitelisted_companies_count: summary.whitelistedCompaniesCount,
    blacklisted_companies_count: summary.blacklistedCompaniesCount,
  };
}

function buildEvidencePayload(entries: LlmContextEvidenceEntry[]) {
  return entries.map((entry) => ({
    type: entry.type,
    label: entry.label,
  }));
}

function buildFeedbackStatePayload(feedbackState?: FeedbackStateInput | null) {
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

export function buildProfileInsightsPayload(context: LlmContext) {
  return {
    profile_id: context.profileId,
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
}

export function mapProfileInsightsResponse(response: MlProfileInsightsResponse) {
  return {
    profileSummary: response.profile_summary,
    searchStrategySummary: response.search_strategy_summary,
    strengths: response.strengths,
    risks: response.risks,
    recommendedActions: response.recommended_actions,
    topFocusAreas: response.top_focus_areas,
    searchTermSuggestions: response.search_term_suggestions,
    applicationStrategy: response.application_strategy,
  };
}

export function buildWeeklyGuidancePayload(payload: {
  profileId: string;
  analyticsSummary: AnalyticsSummary;
  behaviorSummary: BehaviorSummary;
  funnelSummary: FunnelSummary;
  llmContext: LlmContext;
}) {
  return {
    profile_id: payload.profileId,
    analytics_summary: {
      feedback: buildFeedbackSummaryPayload(payload.analyticsSummary.feedback),
      jobs_by_source: payload.analyticsSummary.jobsBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      jobs_by_lifecycle: {
        total: payload.analyticsSummary.jobsByLifecycle.total,
        active: payload.analyticsSummary.jobsByLifecycle.active,
        inactive: payload.analyticsSummary.jobsByLifecycle.inactive,
        reactivated: payload.analyticsSummary.jobsByLifecycle.reactivated,
      },
      top_matched_roles: payload.analyticsSummary.topMatchedRoles,
      top_matched_skills: payload.analyticsSummary.topMatchedSkills,
      top_matched_keywords: payload.analyticsSummary.topMatchedKeywords,
    },
    behavior_summary: {
      search_run_count: payload.behaviorSummary.searchRunCount,
      top_positive_sources: payload.behaviorSummary.topPositiveSources.map(
        (signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        }),
      ),
      top_negative_sources: payload.behaviorSummary.topNegativeSources.map(
        (signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        }),
      ),
      top_positive_role_families:
        payload.behaviorSummary.topPositiveRoleFamilies.map((signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        })),
      top_negative_role_families:
        payload.behaviorSummary.topNegativeRoleFamilies.map((signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        })),
      source_signal_counts: payload.behaviorSummary.sourceSignalCounts.map(
        (signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        }),
      ),
      role_family_signal_counts:
        payload.behaviorSummary.roleFamilySignalCounts.map((signal) => ({
          key: signal.key,
          save_count: signal.saveCount,
          hide_count: signal.hideCount,
          bad_fit_count: signal.badFitCount,
          application_created_count: signal.applicationCreatedCount,
          positive_count: signal.positiveCount,
          negative_count: signal.negativeCount,
          net_score: signal.netScore,
        })),
    },
    funnel_summary: {
      impression_count: payload.funnelSummary.impressionCount,
      open_count: payload.funnelSummary.openCount,
      save_count: payload.funnelSummary.saveCount,
      hide_count: payload.funnelSummary.hideCount,
      bad_fit_count: payload.funnelSummary.badFitCount,
      application_created_count: payload.funnelSummary.applicationCreatedCount,
      fit_explanation_requested_count:
        payload.funnelSummary.fitExplanationRequestedCount,
      application_coach_requested_count:
        payload.funnelSummary.applicationCoachRequestedCount,
      cover_letter_draft_requested_count:
        payload.funnelSummary.coverLetterDraftRequestedCount,
      interview_prep_requested_count:
        payload.funnelSummary.interviewPrepRequestedCount,
      conversion_rates: {
        open_rate_from_impressions:
          payload.funnelSummary.conversionRates.openRateFromImpressions,
        save_rate_from_opens: payload.funnelSummary.conversionRates.saveRateFromOpens,
        application_rate_from_saves:
          payload.funnelSummary.conversionRates.applicationRateFromSaves,
      },
      impressions_by_source: payload.funnelSummary.impressionsBySource.map(
        (entry) => ({
          source: entry.source,
          count: entry.count,
        }),
      ),
      opens_by_source: payload.funnelSummary.opensBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      saves_by_source: payload.funnelSummary.savesBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      applications_by_source: payload.funnelSummary.applicationsBySource.map(
        (entry) => ({
          source: entry.source,
          count: entry.count,
        }),
      ),
    },
    llm_context: buildProfileInsightsPayload(payload.llmContext),
  };
}

export function mapWeeklyGuidanceResponse(response: MlWeeklyGuidanceResponse) {
  return {
    weeklySummary: response.weekly_summary,
    whatIsWorking: response.what_is_working,
    whatIsNotWorking: response.what_is_not_working,
    recommendedSearchAdjustments: response.recommended_search_adjustments,
    recommendedSourceMoves: response.recommended_source_moves,
    recommendedRoleFocus: response.recommended_role_focus,
    funnelBottlenecks: response.funnel_bottlenecks,
    nextWeekPlan: response.next_week_plan,
  };
}

export function buildJobFitExplanationPayload(payload: {
  profileId: string;
  analyzedProfile: AnalyzedProfileInput | null;
  searchProfile: SearchProfileInput | null;
  rankedJob: JobPosting;
  deterministicFit: DeterministicFitInput;
  feedbackState?: FeedbackStateInput | null;
}) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
  };
}

export function mapJobFitExplanationResponse(
  response: MlJobFitExplanationResponse,
) {
  return {
    fitSummary: response.fit_summary,
    whyItMatches: response.why_it_matches,
    risks: response.risks,
    missingSignals: response.missing_signals,
    recommendedNextStep: response.recommended_next_step,
    applicationAngle: response.application_angle,
  };
}

export function buildApplicationCoachPayload(payload: {
  profileId: string;
  analyzedProfile: AnalyzedProfileInput | null;
  searchProfile: SearchProfileInput | null;
  rankedJob: JobPosting;
  deterministicFit: DeterministicFitInput;
  jobFitExplanation?: JobFitExplanationInput | null;
  feedbackState?: FeedbackStateInput | null;
  rawProfileText?: string | null;
}) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: payload.jobFitExplanation
      ? {
          fit_summary: payload.jobFitExplanation.fitSummary,
          why_it_matches: payload.jobFitExplanation.whyItMatches,
          risks: payload.jobFitExplanation.risks,
          missing_signals: payload.jobFitExplanation.missingSignals,
          recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
          application_angle: payload.jobFitExplanation.applicationAngle,
        }
      : null,
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapApplicationCoachResponse(response: MlApplicationCoachResponse) {
  return {
    applicationSummary: response.application_summary,
    resumeFocusPoints: response.resume_focus_points,
    suggestedBullets: response.suggested_bullets,
    coverLetterAngles: response.cover_letter_angles,
    interviewFocus: response.interview_focus,
    gapsToAddress: response.gaps_to_address,
    redFlags: response.red_flags,
  };
}

export function buildCoverLetterDraftPayload(payload: {
  profileId: string;
  analyzedProfile: AnalyzedProfileInput | null;
  searchProfile: SearchProfileInput | null;
  rankedJob: JobPosting;
  deterministicFit: DeterministicFitInput;
  jobFitExplanation?: JobFitExplanationInput | null;
  applicationCoach?: ApplicationCoachInput | null;
  feedbackState?: FeedbackStateInput | null;
  rawProfileText?: string | null;
}) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: payload.jobFitExplanation
      ? {
          fit_summary: payload.jobFitExplanation.fitSummary,
          why_it_matches: payload.jobFitExplanation.whyItMatches,
          risks: payload.jobFitExplanation.risks,
          missing_signals: payload.jobFitExplanation.missingSignals,
          recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
          application_angle: payload.jobFitExplanation.applicationAngle,
        }
      : null,
    application_coach: payload.applicationCoach
      ? {
          application_summary: payload.applicationCoach.applicationSummary,
          resume_focus_points: payload.applicationCoach.resumeFocusPoints,
          suggested_bullets: payload.applicationCoach.suggestedBullets,
          cover_letter_angles: payload.applicationCoach.coverLetterAngles,
          interview_focus: payload.applicationCoach.interviewFocus,
          gaps_to_address: payload.applicationCoach.gapsToAddress,
          red_flags: payload.applicationCoach.redFlags,
        }
      : null,
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapCoverLetterDraftResponse(response: MlCoverLetterDraftResponse) {
  return {
    draftSummary: response.draft_summary,
    openingParagraph: response.opening_paragraph,
    bodyParagraphs: response.body_paragraphs,
    closingParagraph: response.closing_paragraph,
    keyClaimsUsed: response.key_claims_used,
    evidenceGaps: response.evidence_gaps,
    toneNotes: response.tone_notes,
  };
}

export function buildInterviewPrepPayload(payload: {
  profileId: string;
  analyzedProfile: AnalyzedProfileInput | null;
  searchProfile: SearchProfileInput | null;
  rankedJob: JobPosting;
  deterministicFit: DeterministicFitInput;
  jobFitExplanation?: JobFitExplanationInput | null;
  applicationCoach?: ApplicationCoachInput | null;
  coverLetterDraft?: CoverLetterDraftInput | null;
  feedbackState?: FeedbackStateInput | null;
  rawProfileText?: string | null;
}) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: payload.jobFitExplanation
      ? {
          fit_summary: payload.jobFitExplanation.fitSummary,
          why_it_matches: payload.jobFitExplanation.whyItMatches,
          risks: payload.jobFitExplanation.risks,
          missing_signals: payload.jobFitExplanation.missingSignals,
          recommended_next_step: payload.jobFitExplanation.recommendedNextStep,
          application_angle: payload.jobFitExplanation.applicationAngle,
        }
      : null,
    application_coach: payload.applicationCoach
      ? {
          application_summary: payload.applicationCoach.applicationSummary,
          resume_focus_points: payload.applicationCoach.resumeFocusPoints,
          suggested_bullets: payload.applicationCoach.suggestedBullets,
          cover_letter_angles: payload.applicationCoach.coverLetterAngles,
          interview_focus: payload.applicationCoach.interviewFocus,
          gaps_to_address: payload.applicationCoach.gapsToAddress,
          red_flags: payload.applicationCoach.redFlags,
        }
      : null,
    cover_letter_draft: payload.coverLetterDraft
      ? {
          draft_summary: payload.coverLetterDraft.draftSummary,
          opening_paragraph: payload.coverLetterDraft.openingParagraph,
          body_paragraphs: payload.coverLetterDraft.bodyParagraphs,
          closing_paragraph: payload.coverLetterDraft.closingParagraph,
          key_claims_used: payload.coverLetterDraft.keyClaimsUsed,
          evidence_gaps: payload.coverLetterDraft.evidenceGaps,
          tone_notes: payload.coverLetterDraft.toneNotes,
        }
      : null,
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapInterviewPrepResponse(response: MlInterviewPrepResponse) {
  return {
    prepSummary: response.prep_summary,
    likelyTopics: response.likely_topics,
    technicalFocus: response.technical_focus,
    behavioralFocus: response.behavioral_focus,
    storiesToPrepare: response.stories_to_prepare,
    questionsToAsk: response.questions_to_ask,
    riskAreas: response.risk_areas,
    followUpPlan: response.follow_up_plan,
  };
}
