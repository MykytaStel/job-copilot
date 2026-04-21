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

export type ProfileInsights = {
  profileSummary: string;
  searchStrategySummary: string;
  strengths: string[];
  risks: string[];
  recommendedActions: string[];
  topFocusAreas: string[];
  searchTermSuggestions: string[];
  applicationStrategy: string[];
};

export type JobFitExplanation = {
  fitSummary: string;
  whyItMatches: string[];
  risks: string[];
  missingSignals: string[];
  recommendedNextStep: string;
  applicationAngle: string;
};

export type ApplicationCoach = {
  applicationSummary: string;
  resumeFocusPoints: string[];
  suggestedBullets: string[];
  coverLetterAngles: string[];
  interviewFocus: string[];
  gapsToAddress: string[];
  redFlags: string[];
};

export type CoverLetterDraft = {
  draftSummary: string;
  openingParagraph: string;
  bodyParagraphs: string[];
  closingParagraph: string;
  keyClaimsUsed: string[];
  evidenceGaps: string[];
  toneNotes: string[];
};

export type InterviewPrep = {
  prepSummary: string;
  likelyTopics: string[];
  technicalFocus: string[];
  behavioralFocus: string[];
  storiesToPrepare: string[];
  questionsToAsk: string[];
  riskAreas: string[];
  followUpPlan: string[];
};

export type WeeklyGuidance = {
  weeklySummary: string;
  whatIsWorking: string[];
  whatIsNotWorking: string[];
  recommendedSearchAdjustments: string[];
  recommendedSourceMoves: string[];
  recommendedRoleFocus: string[];
  funnelBottlenecks: string[];
  nextWeekPlan: string[];
};
