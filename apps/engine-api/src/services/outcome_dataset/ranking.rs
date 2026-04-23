use crate::domain::matching::JobFit;

use super::types::OutcomeRankingFeatures;

pub(super) fn ranking_features(
    fit: JobFit,
    behavior_score_delta: i16,
    behavior_score: u8,
    behavior_reasons: Vec<String>,
    learned_reranker_score_delta: i16,
    learned_reranker_score: u8,
    learned_reasons: Vec<String>,
) -> OutcomeRankingFeatures {
    let matched_roles = fit
        .matched_roles
        .into_iter()
        .map(|role| role.to_string())
        .collect::<Vec<_>>();
    let matched_role_count = matched_roles.len();
    let matched_skill_count = fit.matched_skills.len();
    let matched_keyword_count = fit.matched_keywords.len();

    OutcomeRankingFeatures {
        deterministic_score: fit.score,
        behavior_score_delta,
        behavior_score,
        learned_reranker_score_delta,
        learned_reranker_score,
        matched_roles,
        matched_skills: fit.matched_skills,
        matched_keywords: fit.matched_keywords,
        matched_role_count,
        matched_skill_count,
        matched_keyword_count,
        fit_reasons: fit.reasons,
        behavior_reasons,
        learned_reasons,
    }
}
