use serde::{Deserialize, Deserializer};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::application::model::{
    ApplicationOutcome, CreateActivity, CreateApplication, CreateApplicationContact, CreateContact,
    CreateNote, UpdateApplication, UpsertOffer,
};

#[derive(Default, Deserialize)]
pub struct CreateApplicationRequest {
    pub job_id: String,
    pub status: String,
    pub applied_at: Option<String>,
    #[serde(default)]
    pub profile_id: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct UpdateApplicationRequest {
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_due_date")]
    pub due_date: Option<Option<String>>,
    pub outcome: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_outcome_date")]
    pub outcome_date: Option<Option<String>>,
    pub rejection_stage: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateActivityRequest {
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
}

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateContactRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateApplicationContactRequest {
    pub contact_id: String,
    pub relationship: String,
}

#[derive(Deserialize)]
pub struct UpsertOfferRequest {
    pub status: String,
    pub compensation_min: Option<i32>,
    pub compensation_max: Option<i32>,
    pub compensation_currency: Option<String>,
    pub starts_at: Option<String>,
    pub notes: Option<String>,
}

pub struct ValidatedCreateApplicationRequest {
    pub application: CreateApplication,
}

impl CreateApplicationRequest {
    pub fn validate(self) -> Result<ValidatedCreateApplicationRequest, ApiError> {
        let profile_id = validate_optional_trimmed(self.profile_id);
        Ok(ValidatedCreateApplicationRequest {
            application: CreateApplication {
                profile_id: profile_id.clone(),
                job_id: validate_required("job_id", self.job_id)?,
                status: validate_status(self.status)?,
                applied_at: self.applied_at,
            },
        })
    }
}

impl CreateContactRequest {
    pub fn validate(self) -> Result<CreateContact, ApiError> {
        Ok(CreateContact {
            name: validate_required("name", self.name)?,
            email: validate_optional_email(self.email)?,
            phone: validate_optional_trimmed(self.phone),
            linkedin_url: validate_optional_trimmed(self.linkedin_url),
            company: validate_optional_trimmed(self.company),
            role: validate_optional_trimmed(self.role),
        })
    }
}

impl CreateApplicationContactRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateApplicationContact, ApiError> {
        Ok(CreateApplicationContact {
            application_id: application_id.to_string(),
            contact_id: validate_required("contact_id", self.contact_id)?,
            relationship: validate_relationship(self.relationship)?,
        })
    }
}

impl UpdateApplicationRequest {
    pub fn validate(self) -> Result<UpdateApplication, ApiError> {
        if self.status.is_none()
            && self.due_date.is_none()
            && self.outcome.is_none()
            && self.outcome_date.is_none()
            && self.rejection_stage.is_none()
        {
            return Err(ApiError::bad_request_with_details(
                "empty_application_patch",
                "PATCH /applications/:id requires at least one field",
                json!({
                    "allowed_fields": ["status", "due_date", "outcome", "outcome_date", "rejection_stage"]
                }),
            ));
        }

        let outcome = self.outcome.map(validate_outcome).transpose()?;
        let rejection_stage = self
            .rejection_stage
            .as_deref()
            .map(validate_rejection_stage)
            .transpose()?
            .map(str::to_string);

        Ok(UpdateApplication {
            status: self.status.map(validate_status).transpose()?,
            due_date: self.due_date,
            outcome: outcome.map(Some),
            outcome_date: self.outcome_date,
            rejection_stage: rejection_stage.map(Some),
        })
    }
}

impl CreateNoteRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateNote, ApiError> {
        let content = validate_required("content", self.content)?;
        Ok(CreateNote {
            application_id: application_id.to_string(),
            content,
        })
    }
}

impl UpsertOfferRequest {
    pub fn validate(self, application_id: &str) -> Result<UpsertOffer, ApiError> {
        if let (Some(min), Some(max)) = (self.compensation_min, self.compensation_max) {
            if min > max {
                return Err(ApiError::bad_request_with_details(
                    "invalid_offer_input",
                    "Field 'compensation_min' must be less than or equal to 'compensation_max'",
                    json!({
                        "field": "compensation_min",
                        "compensation_min": min,
                        "compensation_max": max,
                    }),
                ));
            }
        }

        Ok(UpsertOffer {
            application_id: application_id.to_string(),
            status: validate_offer_status(self.status)?,
            compensation_min: self.compensation_min,
            compensation_max: self.compensation_max,
            compensation_currency: validate_optional_trimmed(self.compensation_currency),
            starts_at: validate_optional_trimmed(self.starts_at),
            notes: validate_optional_trimmed(self.notes),
        })
    }
}

impl CreateActivityRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateActivity, ApiError> {
        let activity_type = validate_required("activity_type", self.activity_type)?;
        let description = validate_required("description", self.description)?;
        let happened_at = validate_required("happened_at", self.happened_at)?;

        if !matches!(
            activity_type.as_str(),
            "interview" | "call" | "email" | "offer" | "rejection" | "other"
        ) {
            return Err(ApiError::bad_request_with_details(
                "invalid_activity_type",
                "Unsupported activity type",
                json!({
                    "field": "activity_type",
                    "allowed_values": ["interview", "call", "email", "offer", "rejection", "other"],
                    "received": activity_type,
                }),
            ));
        }

        Ok(CreateActivity {
            application_id: application_id.to_string(),
            activity_type,
            description,
            happened_at,
        })
    }
}

fn validate_required(field: &'static str, value: String) -> Result<String, ApiError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_input",
            format!("Field '{field}' must not be empty"),
            json!({ "field": field }),
        ));
    }

    Ok(value)
}

fn validate_optional_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

fn deserialize_patch_due_date<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(deserializer)?))
}

fn deserialize_patch_outcome_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(deserializer)?))
}

fn validate_optional_email(value: Option<String>) -> Result<Option<String>, ApiError> {
    validate_optional_trimmed(value)
        .map(validate_email)
        .transpose()
}

fn validate_email(value: String) -> Result<String, ApiError> {
    let value = validate_required("email", value)?;

    if !value.contains('@') {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_input",
            "Field 'email' must contain '@'",
            json!({ "field": "email" }),
        ));
    }

    Ok(value)
}

fn validate_relationship(value: String) -> Result<String, ApiError> {
    let value = validate_required("relationship", value)?;

    if !matches!(
        value.as_str(),
        "recruiter" | "hiring_manager" | "interviewer" | "referrer" | "other"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_contact_relationship",
            "Unsupported contact relationship",
            json!({
                "field": "relationship",
                "allowed_values": ["recruiter", "hiring_manager", "interviewer", "referrer", "other"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_offer_status(value: String) -> Result<String, ApiError> {
    let value = validate_required("status", value)?;

    if !matches!(
        value.as_str(),
        "draft" | "received" | "accepted" | "declined" | "expired"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_offer_status",
            "Unsupported offer status",
            json!({
                "field": "status",
                "allowed_values": ["draft", "received", "accepted", "declined", "expired"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_status(value: String) -> Result<String, ApiError> {
    let value = validate_required("status", value)?;

    if !matches!(
        value.as_str(),
        "saved" | "applied" | "interview" | "offer" | "rejected"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_status",
            "Unsupported application status",
            json!({
                "field": "status",
                "allowed_values": ["saved", "applied", "interview", "offer", "rejected"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_outcome(value: String) -> Result<ApplicationOutcome, ApiError> {
    let value = validate_required("outcome", value)?;
    ApplicationOutcome::parse(&value).ok_or_else(|| {
        ApiError::bad_request_with_details(
            "invalid_application_outcome",
            "Unsupported application outcome",
            json!({
                "field": "outcome",
                "allowed_values": [
                    "phone_screen", "technical_interview", "final_interview",
                    "offer_received", "rejected", "ghosted", "withdrew"
                ],
                "received": value,
            }),
        )
    })
}

fn validate_rejection_stage(value: &str) -> Result<&str, ApiError> {
    if !matches!(
        value.trim(),
        "applied" | "phone_screen" | "technical_interview" | "final_interview"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_rejection_stage",
            "Unsupported rejection stage",
            json!({
                "field": "rejection_stage",
                "allowed_values": ["applied", "phone_screen", "technical_interview", "final_interview"],
                "received": value,
            }),
        ));
    }

    Ok(value.trim())
}
