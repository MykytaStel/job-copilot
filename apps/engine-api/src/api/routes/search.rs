#[path = "search/comparison.rs"]
mod comparison;
#[path = "search/handlers.rs"]
mod handlers;
#[path = "search/reranking.rs"]
mod reranking;

use crate::services::search_ranking::RankedJob;

use comparison::{build_reranker_comparison, mark_reranker_fallback};
#[cfg(test)]
pub use handlers::SearchQuery;
pub use handlers::{run_search, search};
pub(crate) use reranking::{
    SearchLearningAggregates, apply_behavior_scoring, apply_feedback_scoring,
    apply_learned_reranking, apply_salary_scoring, apply_trained_reranking,
    load_learning_aggregates, load_search_salary_expectation, score_by_job_id,
};

fn sort_ranked_jobs(ranked_jobs: &mut [RankedJob]) {
    ranked_jobs.sort_by(|left, right| {
        right
            .fit
            .score
            .cmp(&left.fit.score)
            .then_with(|| right.job.job.last_seen_at.cmp(&left.job.job.last_seen_at))
            .then_with(|| left.job.job.id.cmp(&right.job.job.id))
    });
}

#[cfg(test)]
#[path = "search/tests.rs"]
mod tests;
