use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote,
    ApplicationOutcome, Contact, Offer, Task,
};
use crate::domain::job::model::Job;
use crate::domain::resume::model::ResumeVersion;
use crate::domain::search::global::ApplicationSearchHit;

use super::rows::{
    ApplicationDetailRow, ApplicationRow, ApplicationSearchHitRow, ContactJoinRow, ContactRow,
    OfferRow,
};

impl From<ApplicationRow> for Application {
    fn from(row: ApplicationRow) -> Self {
        Self {
            id: row.id,
            job_id: row.job_id,
            resume_id: row.resume_id,
            status: row.status,
            applied_at: row.applied_at,
            due_date: row.due_date,
            outcome: row.outcome.as_deref().and_then(ApplicationOutcome::parse),
            outcome_date: row.outcome_date,
            rejection_stage: row.rejection_stage,
            updated_at: row.updated_at,
        }
    }
}

impl From<ApplicationSearchHitRow> for ApplicationSearchHit {
    fn from(row: ApplicationSearchHitRow) -> Self {
        Self {
            id: row.id,
            job_id: row.job_id,
            resume_id: row.resume_id,
            status: row.status,
            applied_at: row.applied_at,
            due_date: row.due_date,
            updated_at: row.updated_at,
            job_title: row.job_title,
            company_name: row.company_name,
        }
    }
}

impl TryFrom<(ApplicationDetailRow, Option<Offer>)> for ApplicationDetail {
    type Error = RepositoryError;

    fn try_from(
        (row, offer): (ApplicationDetailRow, Option<Offer>),
    ) -> Result<Self, Self::Error> {
        let resume = match row.resume_version {
            None => None,
            Some(version) => Some(ResumeVersion {
                id: row.application_resume_id.clone().ok_or_else(|| {
                    RepositoryError::InvalidData {
                        message: "resume_id missing on joined resume row".into(),
                    }
                })?,
                version,
                filename: row
                    .resume_filename
                    .ok_or_else(|| RepositoryError::InvalidData {
                        message: "resume filename missing on joined resume row".into(),
                    })?,
                raw_text: row
                    .resume_raw_text
                    .ok_or_else(|| RepositoryError::InvalidData {
                        message: "resume raw_text missing on joined resume row".into(),
                    })?,
                is_active: row
                    .resume_is_active
                    .ok_or_else(|| RepositoryError::InvalidData {
                        message: "resume is_active missing on joined resume row".into(),
                    })?,
                uploaded_at: row.resume_uploaded_at.ok_or_else(|| {
                    RepositoryError::InvalidData {
                        message: "resume uploaded_at missing on joined resume row".into(),
                    }
                })?,
            }),
        };

        let notes = row
            .notes_json
            .0
            .into_iter()
            .map(|r| ApplicationNote {
                id: r.id,
                application_id: r.application_id,
                content: r.content,
                created_at: r.created_at,
            })
            .collect();

        let contacts = row
            .contacts_json
            .0
            .into_iter()
            .map(|r| ApplicationContact {
                id: r.id,
                application_id: r.application_id,
                relationship: r.relationship,
                contact: Contact {
                    id: r.contact_id,
                    name: r.contact_name,
                    email: r.contact_email,
                    phone: r.contact_phone,
                    linkedin_url: r.contact_linkedin_url,
                    company: r.contact_company,
                    role: r.contact_role,
                    created_at: r.contact_created_at,
                },
            })
            .collect();

        let activities = row
            .activities_json
            .0
            .into_iter()
            .map(|r| Activity {
                id: r.id,
                application_id: r.application_id,
                activity_type: r.activity_type,
                description: r.description,
                happened_at: r.happened_at,
                created_at: r.created_at,
            })
            .collect();

        let tasks = row
            .tasks_json
            .0
            .into_iter()
            .map(|r| Task {
                id: r.id,
                application_id: r.application_id,
                title: r.title,
                remind_at: r.remind_at,
                done: r.done,
                created_at: r.created_at,
            })
            .collect();

        Ok(Self {
            application: Application {
                id: row.application_id,
                job_id: row.application_job_id,
                resume_id: row.application_resume_id,
                status: row.application_status,
                applied_at: row.application_applied_at,
                due_date: row.application_due_date,
                outcome: row
                    .application_outcome
                    .as_deref()
                    .and_then(ApplicationOutcome::parse),
                outcome_date: row.application_outcome_date,
                rejection_stage: row.application_rejection_stage,
                updated_at: row.application_updated_at,
            },
            job: Job {
                id: row.job_id,
                title: row.job_title,
                company_name: row.job_company_name,
                location: None,
                remote_type: row.job_remote_type,
                seniority: row.job_seniority,
                description_text: row.job_description_text,
                salary_min: row.job_salary_min,
                salary_max: row.job_salary_max,
                salary_currency: row.job_salary_currency,
                posted_at: row.job_posted_at,
                last_seen_at: row.job_last_seen_at,
                is_active: row.job_is_active,
            },
            resume,
            offer,
            notes,
            contacts,
            activities,
            tasks,
        })
    }
}

impl From<ContactRow> for Contact {
    fn from(row: ContactRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            phone: row.phone,
            linkedin_url: row.linkedin_url,
            company: row.company,
            role: row.role,
            created_at: row.created_at,
        }
    }
}

impl From<ContactJoinRow> for ApplicationContact {
    fn from(row: ContactJoinRow) -> Self {
        Self {
            id: row.id,
            application_id: row.application_id,
            relationship: row.relationship,
            contact: Contact {
                id: row.contact_id,
                name: row.contact_name,
                email: row.contact_email,
                phone: row.contact_phone,
                linkedin_url: row.contact_linkedin_url,
                company: row.contact_company,
                role: row.contact_role,
                created_at: row.contact_created_at,
            },
        }
    }
}

impl From<OfferRow> for Offer {
    fn from(row: OfferRow) -> Self {
        Self {
            id: row.id,
            application_id: row.application_id,
            status: row.status,
            compensation_min: row.compensation_min,
            compensation_max: row.compensation_max,
            compensation_currency: row.compensation_currency,
            starts_at: row.starts_at,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
