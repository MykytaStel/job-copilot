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
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub application_created: bool,
    pub saved_count: usize,
    pub hidden_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
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
            saved: value.saved,
            hidden: value.hidden,
            bad_fit: value.bad_fit,
            application_created: value.application_created,
            saved_count: value.saved_count,
            hidden_count: value.hidden_count,
            bad_fit_count: value.bad_fit_count,
            application_created_count: value.application_created_count,
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
