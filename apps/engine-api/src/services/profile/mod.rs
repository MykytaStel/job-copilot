//! Profile analysis pipeline over raw profile text.
//!
//! This module owns candidate interpretation: signal extraction, canonical role
//! inference, search-term suggestions, and presentation-friendly summaries.
//! Persisted profile CRUD stays in `crate::services::profile_records`.

pub(crate) mod matching;
mod presentation;
pub(crate) mod rules;
pub mod service;
#[cfg(test)]
mod tests;
