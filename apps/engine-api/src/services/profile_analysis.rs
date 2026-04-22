//! Explicit raw-profile analysis boundary.
//!
//! This module owns candidate interpretation from raw profile text. Stored
//! profile CRUD lives in `crate::services::profile_records`.

pub(crate) use crate::services::profile::matching as text;
pub(crate) use crate::services::profile::rules;
pub use crate::services::profile::service::ProfileAnalysisService;
