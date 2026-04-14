use serde::{Deserialize, Serialize};

use crate::domain::role::RoleId;
use crate::domain::source::SourceId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRegion {
    Ua,
    Eu,
    EuRemote,
    Poland,
    Germany,
    Uk,
    Us,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkMode {
    Remote,
    Hybrid,
    Onsite,
}

#[derive(Clone, Debug, Default)]
pub struct SearchPreferences {
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub preferred_roles: Vec<RoleId>,
    pub allowed_sources: Vec<SourceId>,
    pub include_keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SearchProfile {
    pub primary_role: RoleId,
    pub target_roles: Vec<RoleId>,
    pub seniority: String,
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub allowed_sources: Vec<SourceId>,
    pub search_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
}
