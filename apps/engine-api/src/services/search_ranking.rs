//! Explicit search/feed ranking boundary.
//!
//! This module covers deterministic search ranking plus search reranker runtime
//! resolution. Single profile/job fit scoring lives in
//! `crate::services::fit_scoring`.

pub(crate) use crate::services::matching::summarize_match_quality;
pub use crate::services::matching::{
    RankedJob, SearchMatchingService as SearchRankingService, SearchRunResult,
};

pub mod runtime {
    pub use crate::services::ranking::runtime::*;
}
