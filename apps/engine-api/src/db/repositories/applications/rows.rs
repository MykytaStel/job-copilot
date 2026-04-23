use sqlx::FromRow;

#[derive(FromRow)]
pub(super) struct ApplicationRow {
    pub(super) id: String,
    pub(super) job_id: String,
    pub(super) resume_id: Option<String>,
    pub(super) status: String,
    pub(super) applied_at: Option<String>,
    pub(super) due_date: Option<String>,
    pub(super) outcome: Option<String>,
    pub(super) outcome_date: Option<String>,
    pub(super) rejection_stage: Option<String>,
    pub(super) updated_at: String,
}

#[derive(FromRow)]
pub(super) struct NoteRow {
    pub(super) id: String,
    pub(super) application_id: String,
    pub(super) content: String,
    pub(super) created_at: String,
}

#[derive(FromRow)]
pub(super) struct ContactRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) email: Option<String>,
    pub(super) phone: Option<String>,
    pub(super) linkedin_url: Option<String>,
    pub(super) company: Option<String>,
    pub(super) role: Option<String>,
    pub(super) created_at: String,
}

#[derive(FromRow)]
pub(super) struct ContactJoinRow {
    pub(super) id: String,
    pub(super) application_id: String,
    pub(super) relationship: String,
    pub(super) contact_id: String,
    pub(super) contact_name: String,
    pub(super) contact_email: Option<String>,
    pub(super) contact_phone: Option<String>,
    pub(super) contact_linkedin_url: Option<String>,
    pub(super) contact_company: Option<String>,
    pub(super) contact_role: Option<String>,
    pub(super) contact_created_at: String,
}

#[derive(FromRow)]
pub(super) struct OfferRow {
    pub(super) id: String,
    pub(super) application_id: String,
    pub(super) status: String,
    pub(super) compensation_min: Option<i32>,
    pub(super) compensation_max: Option<i32>,
    pub(super) compensation_currency: Option<String>,
    pub(super) starts_at: Option<String>,
    pub(super) notes: Option<String>,
    pub(super) created_at: String,
    pub(super) updated_at: String,
}

#[derive(FromRow)]
pub(super) struct ActivityRow {
    pub(super) id: String,
    pub(super) application_id: String,
    pub(super) activity_type: String,
    pub(super) description: String,
    pub(super) happened_at: String,
    pub(super) created_at: String,
}

#[derive(FromRow)]
pub(super) struct TaskRow {
    pub(super) id: String,
    pub(super) application_id: String,
    pub(super) title: String,
    pub(super) remind_at: Option<String>,
    pub(super) done: bool,
    pub(super) created_at: String,
}

#[derive(FromRow)]
pub(super) struct ApplicationDetailRow {
    pub(super) application_id: String,
    pub(super) application_job_id: String,
    pub(super) application_resume_id: Option<String>,
    pub(super) application_status: String,
    pub(super) application_applied_at: Option<String>,
    pub(super) application_due_date: Option<String>,
    pub(super) application_outcome: Option<String>,
    pub(super) application_outcome_date: Option<String>,
    pub(super) application_rejection_stage: Option<String>,
    pub(super) application_updated_at: String,
    pub(super) job_id: String,
    pub(super) job_title: String,
    pub(super) job_company_name: String,
    pub(super) job_remote_type: Option<String>,
    pub(super) job_seniority: Option<String>,
    pub(super) job_description_text: String,
    pub(super) job_salary_min: Option<i32>,
    pub(super) job_salary_max: Option<i32>,
    pub(super) job_salary_currency: Option<String>,
    pub(super) job_posted_at: Option<String>,
    pub(super) job_last_seen_at: String,
    pub(super) job_is_active: bool,
    pub(super) resume_version: Option<i32>,
    pub(super) resume_filename: Option<String>,
    pub(super) resume_raw_text: Option<String>,
    pub(super) resume_is_active: Option<bool>,
    pub(super) resume_uploaded_at: Option<String>,
}

#[derive(FromRow)]
pub(super) struct ApplicationSearchHitRow {
    pub(super) id: String,
    pub(super) job_id: String,
    pub(super) resume_id: Option<String>,
    pub(super) status: String,
    pub(super) applied_at: Option<String>,
    pub(super) due_date: Option<String>,
    pub(super) updated_at: String,
    pub(super) job_title: String,
    pub(super) company_name: String,
}
