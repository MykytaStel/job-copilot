use serde::{Deserialize, Serialize};

use crate::domain::role::RoleId;
use crate::domain::source::SourceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchSalaryExpectation {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub currency: String,
}

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoringWeights {
    #[serde(default = "default_skill_match_importance")]
    pub skill_match_importance: u8,

    #[serde(default = "default_salary_fit_importance")]
    pub salary_fit_importance: u8,

    #[serde(default = "default_job_freshness_importance")]
    pub job_freshness_importance: u8,

    #[serde(default = "default_remote_work_importance")]
    pub remote_work_importance: u8,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            skill_match_importance: default_skill_match_importance(),
            salary_fit_importance: default_salary_fit_importance(),
            job_freshness_importance: default_job_freshness_importance(),
            remote_work_importance: default_remote_work_importance(),
        }
    }
}

impl ScoringWeights {
    pub fn normalized(self) -> Self {
        Self {
            skill_match_importance: clamp_weight(self.skill_match_importance),
            salary_fit_importance: clamp_weight(self.salary_fit_importance),
            job_freshness_importance: clamp_weight(self.job_freshness_importance),
            remote_work_importance: clamp_weight(self.remote_work_importance),
        }
    }

    pub fn skill_match_multiplier(&self) -> f32 {
        f32::from(clamp_weight(self.skill_match_importance))
            / f32::from(default_skill_match_importance())
    }

    pub fn job_freshness_multiplier(&self) -> f32 {
        f32::from(clamp_weight(self.job_freshness_importance))
            / f32::from(default_job_freshness_importance())
    }

    pub fn remote_work_multiplier(&self) -> f32 {
        f32::from(clamp_weight(self.remote_work_importance))
            / f32::from(default_remote_work_importance())
    }

    pub fn salary_fit_multiplier(&self) -> f32 {
        f32::from(clamp_weight(self.salary_fit_importance))
            / f32::from(default_salary_fit_importance())
    }
}

fn clamp_weight(value: u8) -> u8 {
    value.clamp(1, 10)
}

fn default_skill_match_importance() -> u8 {
    8
}

fn default_salary_fit_importance() -> u8 {
    6
}

fn default_job_freshness_importance() -> u8 {
    5
}

fn default_remote_work_importance() -> u8 {
    5
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPreferences {
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub preferred_roles: Vec<RoleId>,
    pub allowed_sources: Vec<SourceId>,
    pub include_keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,

    #[serde(default)]
    pub scoring_weights: ScoringWeights,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRoleCandidate {
    pub role: RoleId,
    pub confidence: u8,
}

#[derive(Clone, Debug)]
pub struct SearchProfile {
    pub primary_role: RoleId,
    pub primary_role_confidence: Option<u8>,
    pub target_roles: Vec<RoleId>,
    pub role_candidates: Vec<SearchRoleCandidate>,
    pub seniority: String,
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub allowed_sources: Vec<SourceId>,
    pub profile_skills: Vec<String>,
    pub profile_keywords: Vec<String>,
    pub search_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
    pub scoring_weights: ScoringWeights,
    pub salary_expectation: Option<SearchSalaryExpectation>,
}
