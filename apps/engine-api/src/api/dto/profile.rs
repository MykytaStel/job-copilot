use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
use crate::domain::profile::model::{
    CreateProfile, Profile, ProfileAnalysis as PersistedProfileAnalysis, UpdateProfile,
};

#[derive(Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
}

#[derive(Default, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub location: Option<Option<String>>,
    pub raw_text: Option<String>,
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
    pub analysis: Option<PersistedProfileAnalysisResponse>,
    pub created_at: String,
    pub updated_at: String,
    pub skills_updated_at: Option<String>,
}

impl CreateProfileRequest {
    pub fn validate(self) -> Result<CreateProfile, ApiError> {
        Ok(CreateProfile {
            name: validate_required_string("name", self.name, 200)?,
            email: validate_email(self.email)?,
            location: validate_optional_string("location", self.location, 200)?,
            raw_text: validate_required_string("raw_text", self.raw_text, 20_000)?,
        })
    }
}

impl UpdateProfileRequest {
    pub fn validate(self) -> Result<UpdateProfile, ApiError> {
        if self.name.is_none()
            && self.email.is_none()
            && self.location.is_none()
            && self.raw_text.is_none()
        {
            return Err(ApiError::bad_request_with_details(
                "empty_profile_patch",
                "PATCH /profiles/:id requires at least one field",
                json!({
                    "allowed_fields": ["name", "email", "location", "raw_text"]
                }),
            ));
        }

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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
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
}
