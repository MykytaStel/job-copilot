//! Explicit single profile/job fit-scoring boundary.
//!
//! Prefer this module when the caller needs a deterministic fit score for one
//! profile/job pair outside the search-ranking pipeline.

pub use crate::services::ranking::RankingService as FitScoringService;
