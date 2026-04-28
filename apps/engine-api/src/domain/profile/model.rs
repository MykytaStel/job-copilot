use crate::domain::role::RoleId;
use crate::domain::search::profile::SearchPreferences;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageProficiency {
    pub language: String,
    pub level: LanguageLevel,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageLevel {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
    Native,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
    pub analysis: Option<ProfileAnalysis>,
    pub years_of_experience: Option<i32>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: String,
    pub languages: Vec<LanguageProficiency>,
    pub preferred_locations: Vec<String>,
    pub work_mode_preference: String,
    pub preferred_language: Option<String>,
    pub search_preferences: Option<SearchPreferences>,
    pub created_at: String,
    pub updated_at: String,
    pub skills_updated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileAnalysis {
    pub summary: String,
    pub primary_role: RoleId,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProfile {
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
    pub years_of_experience: Option<i32>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: String,
    pub languages: Vec<LanguageProficiency>,
    pub preferred_locations: Vec<String>,
    pub work_mode_preference: String,
    pub search_preferences: Option<SearchPreferences>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub location: Option<Option<String>>,
    pub raw_text: Option<String>,
    pub years_of_experience: Option<Option<i32>>,
    pub salary_min: Option<Option<i32>>,
    pub salary_max: Option<Option<i32>>,
    pub salary_currency: Option<String>,
    pub languages: Option<Vec<LanguageProficiency>>,
    pub preferred_locations: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub work_mode_preference: Option<String>,
    pub preferred_language: Option<Option<String>>,
    pub search_preferences: Option<Option<SearchPreferences>>,
}
