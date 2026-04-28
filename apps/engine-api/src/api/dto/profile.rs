use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::dto::search_profile::{SearchPreferencesRequest, SearchPreferencesResponse};
use crate::api::error::ApiError;
use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
use crate::domain::profile::model::{
    CreateProfile, LanguageProficiency, Profile, ProfileAnalysis as PersistedProfileAnalysis,
    UpdateProfile,
};

#[derive(Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
    pub years_of_experience: Option<i32>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub languages: Option<Vec<LanguageProficiency>>,
    pub preferred_locations: Option<Vec<String>>,
    pub work_mode_preference: Option<String>,
    pub search_preferences: Option<SearchPreferencesRequest>,
}

#[derive(Default, Deserialize)]
pub struct UpdateProfileRequest {
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
    pub search_preferences: Option<Option<SearchPreferencesRequest>>,
}

#[derive(Clone, Serialize)]
pub struct RoleCandidateResponse {
    pub role: String,
    pub score: u32,
    pub confidence: u8,
    pub matched_signals: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct AnalyzeProfileResponse {
    pub summary: String,
    pub primary_role: String,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
    pub role_candidates: Vec<RoleCandidateResponse>,
    pub suggested_search_terms: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PersistedProfileAnalysisResponse {
    pub summary: String,
    pub primary_role: String,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: String,
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
    pub preferred_language: Option<String>,
    pub search_preferences: Option<SearchPreferencesResponse>,
    pub analysis: Option<PersistedProfileAnalysisResponse>,
    pub created_at: String,
    pub updated_at: String,
    pub skills_updated_at: Option<String>,
}

impl CreateProfileRequest {
    pub fn validate(self) -> Result<CreateProfile, ApiError> {
        let years_of_experience =
            validate_optional_i32("years_of_experience", self.years_of_experience, 0, 80)?;
        let salary_min = validate_optional_i32("salary_min", self.salary_min, 0, 10_000_000)?;
        let salary_max = validate_optional_i32("salary_max", self.salary_max, 0, 10_000_000)?;
        let salary_currency = validate_salary_currency(self.salary_currency)?;
        let languages = validate_languages(self.languages)?;
        let preferred_locations = validate_preferred_locations(self.preferred_locations)?;
        let work_mode_preference = validate_work_mode_preference(self.work_mode_preference)?;

        validate_salary_bounds(salary_min, salary_max, "salary_min", "salary_max")?;

        Ok(CreateProfile {
            name: validate_required_string("name", self.name, 200)?,
            email: validate_email(self.email)?,
            location: validate_optional_string("location", self.location, 200)?,
            raw_text: validate_required_string("raw_text", self.raw_text, 20_000)?,
            years_of_experience,
            salary_min,
            salary_max,
            salary_currency,
            languages,
            preferred_locations,
            work_mode_preference,
            search_preferences: self
                .search_preferences
                .map(|value| value.validate())
                .transpose()?,
        })
    }
}

impl UpdateProfileRequest {
    pub fn validate(self) -> Result<UpdateProfile, ApiError> {
        if self.name.is_none()
            && self.email.is_none()
            && self.location.is_none()
            && self.raw_text.is_none()
            && self.years_of_experience.is_none()
            && self.salary_min.is_none()
            && self.salary_max.is_none()
            && self.salary_currency.is_none()
            && self.languages.is_none()
            && self.preferred_locations.is_none()
            && self.skills.is_none()
            && self.work_mode_preference.is_none()
            && self.preferred_language.is_none()
            && self.search_preferences.is_none()
        {
            return Err(ApiError::bad_request_with_details(
                "empty_profile_patch",
                "PATCH /profiles/:id requires at least one field",
                json!({
                    "allowed_fields": [
                        "name",
                        "email",
                        "location",
                        "raw_text",
                        "years_of_experience",
                        "salary_min",
                        "salary_max",
                        "salary_currency",
                        "languages",
                        "preferred_locations",
                        "skills",
                        "work_mode_preference",
                        "preferred_language",
                        "search_preferences"
                    ]
                }),
            ));
        }

        let years_of_experience = self
            .years_of_experience
            .map(|value| validate_optional_i32("years_of_experience", value, 0, 80))
            .transpose()?;
        let salary_min = self
            .salary_min
            .map(|value| validate_optional_i32("salary_min", value, 0, 10_000_000))
            .transpose()?;
        let salary_max = self
            .salary_max
            .map(|value| validate_optional_i32("salary_max", value, 0, 10_000_000))
            .transpose()?;
        let salary_currency = self
            .salary_currency
            .map(|value| validate_salary_currency(Some(value)))
            .transpose()?;
        let languages = self
            .languages
            .map(|value| validate_languages(Some(value)))
            .transpose()?;
        let preferred_locations = self
            .preferred_locations
            .map(|value| validate_preferred_locations(Some(value)))
            .transpose()?;
        let skills = self.skills.map(validate_skills).transpose()?;
        let work_mode_preference = self
            .work_mode_preference
            .map(|value| validate_work_mode_preference(Some(value)))
            .transpose()?;
        let preferred_language = self
            .preferred_language
            .map(|value| validate_preferred_language(value))
            .transpose()?;

        validate_salary_bounds(
            salary_min.flatten(),
            salary_max.flatten(),
            "salary_min",
            "salary_max",
        )?;

        Ok(UpdateProfile {
            name: self
                .name
                .map(|value| validate_required_string("name", value, 200))
                .transpose()?,
            email: self.email.map(validate_email).transpose()?,
            location: self
                .location
                .map(|value| validate_optional_string("location", value, 200))
                .transpose()?,
            raw_text: self
                .raw_text
                .map(|value| validate_required_string("raw_text", value, 20_000))
                .transpose()?,
            years_of_experience,
            salary_min,
            salary_max,
            salary_currency,
            languages,
            preferred_locations,
            skills,
            work_mode_preference,
            preferred_language,
            search_preferences: self
                .search_preferences
                .map(|value| value.map(|prefs| prefs.validate()).transpose())
                .transpose()?,
        })
    }
}

impl From<RoleScore> for RoleCandidateResponse {
    fn from(role_score: RoleScore) -> Self {
        Self {
            role: role_score.role.to_string(),
            score: role_score.score,
            confidence: role_score.confidence,
            matched_signals: role_score.matched_signals,
        }
    }
}

impl From<CandidateProfile> for AnalyzeProfileResponse {
    fn from(profile: CandidateProfile) -> Self {
        let CandidateProfile {
            summary,
            primary_role,
            seniority,
            skills,
            keywords,
            role_candidates,
            suggested_search_terms,
        } = profile;

        Self {
            summary,
            primary_role: primary_role.to_string(),
            seniority,
            skills,
            keywords,
            role_candidates: role_candidates
                .into_iter()
                .map(RoleCandidateResponse::from)
                .collect(),
            suggested_search_terms,
        }
    }
}

impl From<PersistedProfileAnalysis> for PersistedProfileAnalysisResponse {
    fn from(analysis: PersistedProfileAnalysis) -> Self {
        Self {
            summary: analysis.summary,
            primary_role: analysis.primary_role.to_string(),
            seniority: analysis.seniority,
            skills: analysis.skills,
            keywords: analysis.keywords,
        }
    }
}

impl From<Profile> for ProfileResponse {
    fn from(profile: Profile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            email: profile.email,
            location: profile.location,
            raw_text: profile.raw_text,
            years_of_experience: profile.years_of_experience,
            salary_min: profile.salary_min,
            salary_max: profile.salary_max,
            salary_currency: profile.salary_currency,
            languages: profile.languages,
            preferred_locations: profile.preferred_locations,
            work_mode_preference: profile.work_mode_preference,
            preferred_language: profile.preferred_language,
            search_preferences: profile
                .search_preferences
                .map(SearchPreferencesResponse::from),
            analysis: profile.analysis.map(PersistedProfileAnalysisResponse::from),
            created_at: profile.created_at,
            updated_at: profile.updated_at,
            skills_updated_at: profile.skills_updated_at,
        }
    }
}

fn validate_required_string(
    field: &'static str,
    value: String,
    max_len: usize,
) -> Result<String, ApiError> {
    let value = value.trim().to_string();

    if value.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            format!("Field '{field}' must not be empty"),
            json!({ "field": field }),
        ));
    }

    if value.len() > max_len {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            format!("Field '{field}' must be at most {max_len} characters"),
            json!({
                "field": field,
                "max_length": max_len,
                "received_length": value.len(),
            }),
        ));
    }

    Ok(value)
}

fn validate_optional_string(
    field: &'static str,
    value: Option<String>,
    max_len: usize,
) -> Result<Option<String>, ApiError> {
    value
        .map(|value| validate_required_string(field, value, max_len))
        .transpose()
}

fn validate_skills(values: Vec<String>) -> Result<Vec<String>, ApiError> {
    let mut result = Vec::new();

    for value in values {
        let skill = validate_required_string("skills", value, 120)?;
        if !result
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&skill))
        {
            result.push(skill);
        }
    }

    if result.len() > 100 {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'skills' must contain at most 100 entries",
            json!({
                "field": "skills",
                "max_items": 100,
                "received_items": result.len(),
            }),
        ));
    }

    Ok(result)
}

fn validate_email(value: String) -> Result<String, ApiError> {
    let value = validate_required_string("email", value, 320)?;

    if !value.contains('@') {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'email' must contain '@'",
            json!({ "field": "email" }),
        ));
    }

    Ok(value)
}

fn validate_optional_i32(
    field: &'static str,
    value: Option<i32>,
    min: i32,
    max: i32,
) -> Result<Option<i32>, ApiError> {
    value
        .map(|value| validate_i32_range(field, value, min, max))
        .transpose()
}

fn validate_i32_range(
    field: &'static str,
    value: i32,
    min: i32,
    max: i32,
) -> Result<i32, ApiError> {
    if value < min || value > max {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            format!("Field '{field}' must be between {min} and {max}"),
            json!({
                "field": field,
                "min": min,
                "max": max,
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_salary_currency(value: Option<String>) -> Result<String, ApiError> {
    let normalized = value
        .unwrap_or_else(|| "USD".to_string())
        .trim()
        .to_uppercase();

    match normalized.as_str() {
        "USD" | "EUR" | "UAH" => Ok(normalized),
        _ => Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'salary_currency' must be one of USD, EUR, UAH",
            json!({
                "field": "salary_currency",
                "allowed_values": ["USD", "EUR", "UAH"],
            }),
        )),
    }
}

fn validate_languages(
    value: Option<Vec<LanguageProficiency>>,
) -> Result<Vec<LanguageProficiency>, ApiError> {
    let mut result = Vec::new();

    for entry in value.unwrap_or_default() {
        let language = validate_required_string("languages.language", entry.language, 80)?;

        if !result
            .iter()
            .any(|existing: &LanguageProficiency| existing.language.eq_ignore_ascii_case(&language))
        {
            result.push(LanguageProficiency {
                language,
                level: entry.level,
            });
        }
    }

    if result.len() > 20 {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'languages' must contain at most 20 entries",
            json!({
                "field": "languages",
                "max_items": 20,
                "received_items": result.len(),
            }),
        ));
    }

    Ok(result)
}

fn validate_preferred_locations(value: Option<Vec<String>>) -> Result<Vec<String>, ApiError> {
    let mut result = Vec::new();

    for value in value.unwrap_or_default() {
        let location = validate_required_string("preferred_locations", value, 120)?;
        if !result
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&location))
        {
            result.push(location);
        }
    }

    if result.len() > 50 {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'preferred_locations' must contain at most 50 entries",
            json!({
                "field": "preferred_locations",
                "max_items": 50,
                "received_items": result.len(),
            }),
        ));
    }

    Ok(result)
}

fn validate_work_mode_preference(value: Option<String>) -> Result<String, ApiError> {
    let normalized = value
        .unwrap_or_else(|| "any".to_string())
        .trim()
        .to_lowercase();

    match normalized.as_str() {
        "remote_only" => Ok("remote_only".to_string()),
        "hybrid" => Ok("hybrid".to_string()),
        "onsite" => Ok("onsite".to_string()),
        "any" => Ok("any".to_string()),
        _ => Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            "Field 'work_mode_preference' must be one of remote_only, hybrid, onsite, any",
            json!({
                "field": "work_mode_preference",
                "allowed_values": ["remote_only", "hybrid", "onsite", "any"],
            }),
        )),
    }
}

fn validate_preferred_language(value: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(raw) = value else { return Ok(None) };
    let normalized = raw.trim().to_lowercase();
    let canonical = match normalized.as_str() {
        "ukrainian" => "Ukrainian",
        "english" => "English",
        "bilingual" => "bilingual",
        "any" => "any",
        _ => {
            return Err(ApiError::bad_request_with_details(
                "invalid_profile_input",
                "Field 'preferred_language' must be one of Ukrainian, English, bilingual, any",
                json!({
                    "field": "preferred_language",
                    "allowed_values": ["Ukrainian", "English", "bilingual", "any"],
                    "received": raw,
                }),
            ));
        }
    };
    Ok(Some(canonical.to_string()))
}

fn validate_salary_bounds(
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    min_field: &'static str,
    max_field: &'static str,
) -> Result<(), ApiError> {
    if let (Some(salary_min), Some(salary_max)) = (salary_min, salary_max)
        && salary_min > salary_max
    {
        return Err(ApiError::bad_request_with_details(
            "invalid_profile_input",
            format!("Field '{min_field}' must be less than or equal to '{max_field}'"),
            json!({
                "field": min_field,
                "related_field": max_field,
                "salary_min": salary_min,
                "salary_max": salary_max,
            }),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use serde_json::json;

    use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
    use crate::domain::profile::model::{LanguageLevel, LanguageProficiency};
    use crate::domain::role::RoleId;

    use super::{AnalyzeProfileResponse, CreateProfileRequest, UpdateProfileRequest};

    #[test]
    fn serializes_role_ids_as_snake_case_strings() {
        let response = AnalyzeProfileResponse::from(CandidateProfile {
            summary: "Summary".to_string(),
            primary_role: RoleId::MobileEngineer,
            seniority: "senior".to_string(),
            skills: vec!["react native".to_string()],
            keywords: vec!["mobile".to_string()],
            role_candidates: vec![RoleScore {
                role: RoleId::MobileEngineer,
                score: 30,
                confidence: 100,
                matched_signals: vec!["react native".to_string()],
            }],
            suggested_search_terms: vec!["mobile engineer".to_string()],
        });

        assert_eq!(response.primary_role, "mobile_engineer");
        assert_eq!(response.role_candidates[0].role, "mobile_engineer");
    }

    #[test]
    fn rejects_invalid_create_payload() {
        let error = CreateProfileRequest {
            name: " ".to_string(),
            email: "invalid".to_string(),
            location: None,
            raw_text: " ".to_string(),
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            languages: None,
            preferred_locations: None,
            work_mode_preference: None,
            search_preferences: None,
        }
        .validate()
        .expect_err("validation should fail");

        assert_eq!(error.into_response().status(), 400);
    }

    #[test]
    fn rejects_empty_patch_payload() {
        let error = UpdateProfileRequest::default()
            .validate()
            .expect_err("validation should fail");

        assert_eq!(error.into_response().status(), 400);
    }

    #[test]
    fn accepts_search_preferences_only_patch() {
        let validated = UpdateProfileRequest {
            search_preferences: Some(Some(
                serde_json::from_value(json!({
                    "target_regions": ["ua"],
                    "work_modes": ["remote"],
                    "preferred_roles": ["frontend_engineer"],
                    "allowed_sources": ["djinni"],
                    "include_keywords": ["product company"],
                    "exclude_keywords": ["gambling"]
                }))
                .expect("payload should deserialize"),
            )),
            ..Default::default()
        }
        .validate()
        .expect("validation should succeed");

        let preferences = validated
            .search_preferences
            .flatten()
            .expect("search preferences should be present");
        assert_eq!(preferences.preferred_roles, vec![RoleId::FrontendEngineer]);
        assert_eq!(preferences.include_keywords, vec!["product company"]);
    }

    #[test]
    fn normalizes_extended_profile_fields() {
        let validated = UpdateProfileRequest {
            years_of_experience: Some(Some(6)),
            salary_min: Some(Some(3000)),
            salary_max: Some(Some(4500)),
            salary_currency: Some("eur".to_string()),
            languages: Some(vec![
                LanguageProficiency {
                    language: "english".to_string(),
                    level: LanguageLevel::C1,
                },
                LanguageProficiency {
                    language: "Polish".to_string(),
                    level: LanguageLevel::B2,
                },
            ]),
            preferred_locations: Some(vec![
                " Kyiv ".to_string(),
                "Remote".to_string(),
                "kyiv".to_string(),
            ]),
            ..Default::default()
        }
        .validate()
        .expect("validation should succeed");

        assert_eq!(validated.years_of_experience, Some(Some(6)));
        assert_eq!(validated.salary_currency.as_deref(), Some("EUR"));
        assert_eq!(
            validated.languages,
            Some(vec![
                LanguageProficiency {
                    language: "english".to_string(),
                    level: LanguageLevel::C1,
                },
                LanguageProficiency {
                    language: "Polish".to_string(),
                    level: LanguageLevel::B2,
                },
            ])
        );
        assert_eq!(
            validated.preferred_locations,
            Some(vec!["Kyiv".to_string(), "Remote".to_string()])
        );
    }
}
