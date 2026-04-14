use serde::Serialize;

use crate::services::behavior::{BehaviorSignalCount, ProfileBehaviorSummary};

#[derive(Debug, Serialize)]
pub struct BehaviorSignalCountResponse {
    pub key: String,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub net_score: i32,
}

#[derive(Debug, Serialize)]
pub struct ProfileBehaviorSummaryResponse {
    pub profile_id: String,
    pub search_run_count: usize,
    pub top_positive_sources: Vec<BehaviorSignalCountResponse>,
    pub top_negative_sources: Vec<BehaviorSignalCountResponse>,
    pub top_positive_role_families: Vec<BehaviorSignalCountResponse>,
    pub top_negative_role_families: Vec<BehaviorSignalCountResponse>,
    pub source_signal_counts: Vec<BehaviorSignalCountResponse>,
    pub role_family_signal_counts: Vec<BehaviorSignalCountResponse>,
}

impl ProfileBehaviorSummaryResponse {
    pub fn from_summary(profile_id: String, summary: ProfileBehaviorSummary) -> Self {
        Self {
            profile_id,
            search_run_count: summary.search_run_count,
            top_positive_sources: summary
                .top_positive_sources
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
            top_negative_sources: summary
                .top_negative_sources
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
            top_positive_role_families: summary
                .top_positive_role_families
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
            top_negative_role_families: summary
                .top_negative_role_families
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
            source_signal_counts: summary
                .source_signal_counts
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
            role_family_signal_counts: summary
                .role_family_signal_counts
                .into_iter()
                .map(BehaviorSignalCountResponse::from)
                .collect(),
        }
    }
}

impl From<BehaviorSignalCount> for BehaviorSignalCountResponse {
    fn from(value: BehaviorSignalCount) -> Self {
        Self {
            key: value.key,
            save_count: value.save_count,
            hide_count: value.hide_count,
            bad_fit_count: value.bad_fit_count,
            application_created_count: value.application_created_count,
            positive_count: value.positive_count,
            negative_count: value.negative_count,
            net_score: value.net_score,
        }
    }
}
