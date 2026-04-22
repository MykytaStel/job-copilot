//! Explicit analyzed-profile -> search-profile derivation boundary.
//!
//! Prefer this import path when the caller needs to build a structured
//! `SearchProfile` from analyzed profile signals and saved preferences.

pub use crate::services::search_profile::service::SearchProfileService as SearchProfileBuilder;
