#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeDataset {
    pub profile_id: String,
    pub label_policy_version: String,
    pub examples: Vec<OutcomeExample>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeExample {
    pub profile_id: String,
    pub job_id: String,
    pub title: String,
    pub company_name: String,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub label_observed_at: Option<String>,
    pub label: OutcomeLabel,
    pub label_score: u8,
    pub label_reasons: Vec<String>,
    pub signals: OutcomeSignals,
    pub ranking: OutcomeRankingFeatures,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeLabel {
    Positive,
    Medium,
    Negative,
}

impl OutcomeLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Medium => "medium",
            Self::Negative => "negative",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeSignals {
    pub viewed: bool,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub applied: bool,
    pub dismissed: bool,
    pub explicit_feedback: bool,
    pub explicit_saved: bool,
    pub explicit_hidden: bool,
    pub explicit_bad_fit: bool,
    pub viewed_event_count: usize,
    pub saved_event_count: usize,
    pub applied_event_count: usize,
    pub dismissed_event_count: usize,
    pub outcome: Option<String>,
    pub reached_interview: bool,
    pub received_offer: bool,
    pub was_rejected: bool,
    pub was_ghosted: bool,
    pub rejection_tags: Vec<String>,
    pub positive_tags: Vec<String>,
    pub has_salary_rejection: bool,
    pub has_remote_rejection: bool,
    pub has_tech_rejection: bool,
    pub salary_signal: Option<String>,
    pub salary_below_expectation: bool,
    pub interest_rating: Option<i8>,
    pub work_mode_deal_breaker: bool,
    pub scrolled_to_bottom: bool,
    pub returned_count: usize,
    pub time_to_apply_days: Option<u32>,
    pub legitimacy_suspicious: bool,
    pub legitimacy_spam: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeRankingFeatures {
    pub deterministic_score: u8,
    pub behavior_score_delta: i16,
    pub behavior_score: u8,
    pub learned_reranker_score_delta: i16,
    pub learned_reranker_score: u8,
    pub matched_roles: Vec<String>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub matched_role_count: usize,
    pub matched_skill_count: usize,
    pub matched_keyword_count: usize,
    pub fit_reasons: Vec<String>,
    pub behavior_reasons: Vec<String>,
    pub learned_reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeDatasetError {
    ProfileAnalysisRequired,
}
