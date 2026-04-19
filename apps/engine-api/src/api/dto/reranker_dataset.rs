use serde::Serialize;

use crate::services::outcome_dataset::{
    OutcomeDataset, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals,
};

#[derive(Debug, Serialize)]
pub struct OutcomeDatasetResponse {
    pub profile_id: String,
    pub label_policy_version: String,
    pub examples: Vec<OutcomeExampleResponse>,
}

#[derive(Debug, Serialize)]
pub struct OutcomeExampleResponse {
    pub profile_id: String,
    pub job_id: String,
    pub title: String,
    pub company_name: String,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub label: String,
    pub label_score: u8,
    pub label_reasons: Vec<String>,
    pub signals: OutcomeSignalsResponse,
    pub ranking: OutcomeRankingFeaturesResponse,
}

#[derive(Debug, Serialize)]
pub struct OutcomeSignalsResponse {
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
}

#[derive(Debug, Serialize)]
pub struct OutcomeRankingFeaturesResponse {
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

impl From<OutcomeDataset> for OutcomeDatasetResponse {
    fn from(value: OutcomeDataset) -> Self {
        Self {
            profile_id: value.profile_id,
            label_policy_version: value.label_policy_version,
            examples: value
                .examples
                .into_iter()
                .map(OutcomeExampleResponse::from)
                .collect(),
        }
    }
}

impl From<OutcomeExample> for OutcomeExampleResponse {
    fn from(value: OutcomeExample) -> Self {
        Self {
            profile_id: value.profile_id,
            job_id: value.job_id,
            title: value.title,
            company_name: value.company_name,
            source: value.source,
            role_family: value.role_family,
            label: value.label.as_str().to_string(),
            label_score: value.label_score,
            label_reasons: value.label_reasons,
            signals: OutcomeSignalsResponse::from(value.signals),
            ranking: OutcomeRankingFeaturesResponse::from(value.ranking),
        }
    }
}

impl From<OutcomeSignals> for OutcomeSignalsResponse {
    fn from(value: OutcomeSignals) -> Self {
        Self {
            viewed: value.viewed,
            saved: value.saved,
            hidden: value.hidden,
            bad_fit: value.bad_fit,
            applied: value.applied,
            dismissed: value.dismissed,
            explicit_feedback: value.explicit_feedback,
            explicit_saved: value.explicit_saved,
            explicit_hidden: value.explicit_hidden,
            explicit_bad_fit: value.explicit_bad_fit,
            viewed_event_count: value.viewed_event_count,
            saved_event_count: value.saved_event_count,
            applied_event_count: value.applied_event_count,
            dismissed_event_count: value.dismissed_event_count,
        }
    }
}

impl From<OutcomeRankingFeatures> for OutcomeRankingFeaturesResponse {
    fn from(value: OutcomeRankingFeatures) -> Self {
        Self {
            deterministic_score: value.deterministic_score,
            behavior_score_delta: value.behavior_score_delta,
            behavior_score: value.behavior_score,
            learned_reranker_score_delta: value.learned_reranker_score_delta,
            learned_reranker_score: value.learned_reranker_score,
            matched_roles: value.matched_roles,
            matched_skills: value.matched_skills,
            matched_keywords: value.matched_keywords,
            matched_role_count: value.matched_role_count,
            matched_skill_count: value.matched_skill_count,
            matched_keyword_count: value.matched_keyword_count,
            fit_reasons: value.fit_reasons,
            behavior_reasons: value.behavior_reasons,
            learned_reasons: value.learned_reasons,
        }
    }
}
