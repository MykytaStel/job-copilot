use crate::domain::role::RoleId;
use crate::domain::search::profile::SearchPreferences;

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
    pub languages: Vec<String>,
    pub preferred_work_mode: Option<String>,
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
    pub languages: Vec<String>,
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
    pub languages: Option<Vec<String>>,
    pub search_preferences: Option<Option<SearchPreferences>>,
}
