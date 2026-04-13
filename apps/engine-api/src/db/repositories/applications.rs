use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateApplication, CreateApplicationContact, CreateContact, CreateNote, Offer, Task,
    UpdateApplication, UpsertOffer,
};
use crate::domain::job::model::Job;
use crate::domain::resume::model::ResumeVersion;

#[derive(Clone)]
pub struct ApplicationsRepository {
    database: Database,
}

#[derive(FromRow)]
struct ApplicationRow {
    id: String,
    job_id: String,
    resume_id: Option<String>,
    status: String,
    applied_at: Option<String>,
    due_date: Option<String>,
    updated_at: String,
}

#[derive(FromRow)]
struct NoteRow {
    id: String,
    application_id: String,
    content: String,
    created_at: String,
}

#[derive(FromRow)]
struct ContactRow {
    id: String,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    linkedin_url: Option<String>,
    company: Option<String>,
    role: Option<String>,
    created_at: String,
}

#[derive(FromRow)]
struct ContactJoinRow {
    id: String,
    application_id: String,
    relationship: String,
    contact_id: String,
    contact_name: String,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    contact_linkedin_url: Option<String>,
    contact_company: Option<String>,
    contact_role: Option<String>,
    contact_created_at: String,
}

#[derive(FromRow)]
struct OfferRow {
    id: String,
    application_id: String,
    status: String,
    compensation_min: Option<i32>,
    compensation_max: Option<i32>,
    compensation_currency: Option<String>,
    starts_at: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(FromRow)]
struct ActivityRow {
    id: String,
    application_id: String,
    activity_type: String,
    description: String,
    happened_at: String,
    created_at: String,
}

#[derive(FromRow)]
struct TaskRow {
    id: String,
    application_id: String,
    title: String,
    remind_at: Option<String>,
    done: bool,
    created_at: String,
}

#[derive(FromRow)]
struct ApplicationDetailRow {
    application_id: String,
    application_job_id: String,
    application_resume_id: Option<String>,
    application_status: String,
    application_applied_at: Option<String>,
    application_due_date: Option<String>,
    application_updated_at: String,
    job_id: String,
    job_title: String,
    job_company_name: String,
    job_remote_type: Option<String>,
    job_seniority: Option<String>,
    job_description_text: String,
    job_salary_min: Option<i32>,
    job_salary_max: Option<i32>,
    job_salary_currency: Option<String>,
    job_posted_at: Option<String>,
    job_last_seen_at: String,
    job_is_active: bool,
    resume_version: Option<i32>,
    resume_filename: Option<String>,
    resume_raw_text: Option<String>,
    resume_is_active: Option<bool>,
    resume_uploaded_at: Option<String>,
}

impl ApplicationsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(
        &self,
        application: &CreateApplication,
    ) -> Result<Application, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            INSERT INTO applications (
                id,
                job_id,
                resume_id,
                status,
                applied_at,
                due_date,
                updated_at
            )
            VALUES ($1, $2, NULL, $3, $4::timestamptz, NULL, NOW())
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&application.job_id)
        .bind(&application.status)
        .bind(&application.applied_at)
        .fetch_one(pool)
        .await?;

        Ok(Application::from(row))
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            SELECT
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            FROM applications
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }

    pub async fn get_detail_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ApplicationDetail>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationDetailRow>(
            r#"
            SELECT
                applications.id AS application_id,
                applications.job_id AS application_job_id,
                applications.resume_id AS application_resume_id,
                applications.status AS application_status,
                applications.applied_at::text AS application_applied_at,
                applications.due_date::text AS application_due_date,
                applications.updated_at::text AS application_updated_at,
                jobs.id AS job_id,
                jobs.title AS job_title,
                jobs.company_name AS job_company_name,
                jobs.remote_type AS job_remote_type,
                jobs.seniority AS job_seniority,
                jobs.description_text AS job_description_text,
                jobs.salary_min AS job_salary_min,
                jobs.salary_max AS job_salary_max,
                jobs.salary_currency AS job_salary_currency,
                jobs.posted_at::text AS job_posted_at,
                jobs.last_seen_at::text AS job_last_seen_at,
                jobs.is_active AS job_is_active,
                resumes.version AS resume_version,
                resumes.filename AS resume_filename,
                resumes.raw_text AS resume_raw_text,
                resumes.is_active AS resume_is_active,
                resumes.uploaded_at::text AS resume_uploaded_at
            FROM applications
            INNER JOIN jobs ON jobs.id = applications.job_id
            LEFT JOIN resumes ON resumes.id = applications.resume_id
            WHERE applications.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let offer = sqlx::query_as::<_, OfferRow>(
            r#"
            SELECT
                id,
                application_id,
                status,
                compensation_min,
                compensation_max,
                compensation_currency,
                starts_at::text AS starts_at,
                notes,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            FROM offers
            WHERE application_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .map(Offer::from);

        let notes: Vec<ApplicationNote> = sqlx::query_as::<_, NoteRow>(
            r#"
            SELECT id, application_id, content, created_at::text AS created_at
            FROM application_notes
            WHERE application_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| ApplicationNote {
            id: r.id,
            application_id: r.application_id,
            content: r.content,
            created_at: r.created_at,
        })
        .collect();

        let contacts: Vec<ApplicationContact> = sqlx::query_as::<_, ContactJoinRow>(
            r#"
            SELECT
                ac.id AS id,
                ac.application_id AS application_id,
                ac.relationship AS relationship,
                c.id AS contact_id,
                c.name AS contact_name,
                c.email AS contact_email,
                c.phone AS contact_phone,
                c.linkedin_url AS contact_linkedin_url,
                c.company AS contact_company,
                c.role AS contact_role,
                c.created_at::text AS contact_created_at
            FROM application_contacts ac
            JOIN contacts c ON c.id = ac.contact_id
            WHERE ac.application_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?
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

        let activities: Vec<Activity> = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT
                id,
                application_id,
                activity_type,
                description,
                happened_at::text AS happened_at,
                created_at::text AS created_at
            FROM activities
            WHERE application_id = $1
            ORDER BY happened_at DESC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?
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

        let tasks: Vec<Task> = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT
                id,
                application_id,
                title,
                remind_at::text AS remind_at,
                done,
                created_at::text AS created_at
            FROM tasks
            WHERE application_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?
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

        Ok(Some(ApplicationDetail::from((
            row, offer, notes, contacts, activities, tasks,
        ))))
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ApplicationRow>(
            r#"
            SELECT
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            FROM applications
            ORDER BY updated_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Application::from).collect())
    }

    pub async fn update(
        &self,
        id: &str,
        update: &UpdateApplication,
    ) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            UPDATE applications
            SET
                status = COALESCE($2, status),
                due_date = CASE
                    WHEN $3 THEN $4::timestamptz
                    ELSE due_date
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(id)
        .bind(&update.status)
        .bind(update.due_date.is_some())
        .bind(&update.due_date)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }

    pub async fn create_note(&self, note: &CreateNote) -> Result<ApplicationNote, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, NoteRow>(
            r#"
            INSERT INTO application_notes (id, application_id, content, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, application_id, content, created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&note.application_id)
        .bind(&note.content)
        .fetch_one(pool)
        .await?;

        Ok(ApplicationNote {
            id: row.id,
            application_id: row.application_id,
            content: row.content,
            created_at: row.created_at,
        })
    }

    pub async fn create_contact(
        &self,
        contact: &CreateContact,
    ) -> Result<Contact, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ContactRow>(
            r#"
            INSERT INTO contacts (
                id,
                name,
                email,
                phone,
                linkedin_url,
                company,
                role,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING
                id,
                name,
                email,
                phone,
                linkedin_url,
                company,
                role,
                created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&contact.name)
        .bind(&contact.email)
        .bind(&contact.phone)
        .bind(&contact.linkedin_url)
        .bind(&contact.company)
        .bind(&contact.role)
        .fetch_one(pool)
        .await?;

        Ok(Contact::from(row))
    }

    pub async fn list_contacts(&self) -> Result<Vec<Contact>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ContactRow>(
            r#"
            SELECT
                id,
                name,
                email,
                phone,
                linkedin_url,
                company,
                role,
                created_at::text AS created_at
            FROM contacts
            ORDER BY created_at DESC, name ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Contact::from).collect())
    }

    pub async fn get_contact_by_id(&self, id: &str) -> Result<Option<Contact>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ContactRow>(
            r#"
            SELECT
                id,
                name,
                email,
                phone,
                linkedin_url,
                company,
                role,
                created_at::text AS created_at
            FROM contacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Contact::from))
    }

    pub async fn attach_contact(
        &self,
        contact: &CreateApplicationContact,
    ) -> Result<ApplicationContact, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ContactJoinRow>(
            r#"
            WITH inserted AS (
                INSERT INTO application_contacts (
                    id,
                    application_id,
                    contact_id,
                    relationship,
                    created_at
                )
                VALUES ($1, $2, $3, $4, NOW())
                RETURNING id, application_id, contact_id, relationship
            )
            SELECT
                inserted.id AS id,
                inserted.application_id AS application_id,
                inserted.relationship AS relationship,
                c.id AS contact_id,
                c.name AS contact_name,
                c.email AS contact_email,
                c.phone AS contact_phone,
                c.linkedin_url AS contact_linkedin_url,
                c.company AS contact_company,
                c.role AS contact_role,
                c.created_at::text AS contact_created_at
            FROM inserted
            JOIN contacts c ON c.id = inserted.contact_id
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&contact.application_id)
        .bind(&contact.contact_id)
        .bind(&contact.relationship)
        .fetch_one(pool)
        .await?;

        Ok(ApplicationContact::from(row))
    }

    pub async fn upsert_offer(&self, offer: &UpsertOffer) -> Result<Offer, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, OfferRow>(
            r#"
            INSERT INTO offers (
                id,
                application_id,
                status,
                compensation_min,
                compensation_max,
                compensation_currency,
                starts_at,
                notes,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8, NOW(), NOW())
            ON CONFLICT (application_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                compensation_min = EXCLUDED.compensation_min,
                compensation_max = EXCLUDED.compensation_max,
                compensation_currency = EXCLUDED.compensation_currency,
                starts_at = EXCLUDED.starts_at,
                notes = EXCLUDED.notes,
                updated_at = NOW()
            RETURNING
                id,
                application_id,
                status,
                compensation_min,
                compensation_max,
                compensation_currency,
                starts_at::text AS starts_at,
                notes,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&offer.application_id)
        .bind(&offer.status)
        .bind(offer.compensation_min)
        .bind(offer.compensation_max)
        .bind(&offer.compensation_currency)
        .bind(&offer.starts_at)
        .bind(&offer.notes)
        .fetch_one(pool)
        .await?;

        Ok(Offer::from(row))
    }

    pub async fn attach_resume(
        &self,
        id: &str,
        resume_id: &str,
    ) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            UPDATE applications
            SET
                resume_id = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(id)
        .bind(resume_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }
}

impl From<ApplicationRow> for Application {
    fn from(row: ApplicationRow) -> Self {
        Self {
            id: row.id,
            job_id: row.job_id,
            resume_id: row.resume_id,
            status: row.status,
            applied_at: row.applied_at,
            due_date: row.due_date,
            updated_at: row.updated_at,
        }
    }
}

impl
    From<(
        ApplicationDetailRow,
        Option<Offer>,
        Vec<ApplicationNote>,
        Vec<ApplicationContact>,
        Vec<Activity>,
        Vec<Task>,
    )> for ApplicationDetail
{
    fn from(
        (row, offer, notes, contacts, activities, tasks): (
            ApplicationDetailRow,
            Option<Offer>,
            Vec<ApplicationNote>,
            Vec<ApplicationContact>,
            Vec<Activity>,
            Vec<Task>,
        ),
    ) -> Self {
        Self {
            application: Application {
                id: row.application_id,
                job_id: row.application_job_id,
                resume_id: row.application_resume_id.clone(),
                status: row.application_status,
                applied_at: row.application_applied_at,
                due_date: row.application_due_date,
                updated_at: row.application_updated_at,
            },
            job: Job {
                id: row.job_id,
                title: row.job_title,
                company_name: row.job_company_name,
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
            resume: row.resume_version.map(|version| ResumeVersion {
                id: row
                    .application_resume_id
                    .expect("resume id should be present when resume is joined"),
                version,
                filename: row
                    .resume_filename
                    .expect("filename should be present when resume is joined"),
                raw_text: row
                    .resume_raw_text
                    .expect("raw_text should be present when resume is joined"),
                is_active: row
                    .resume_is_active
                    .expect("is_active should be present when resume is joined"),
                uploaded_at: row
                    .resume_uploaded_at
                    .expect("uploaded_at should be present when resume is joined"),
            }),
            offer,
            notes,
            contacts,
            activities,
            tasks,
        }
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

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{ApplicationsRepository, RepositoryError};
    use crate::domain::application::model::CreateApplication;

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = ApplicationsRepository::new(Database::disabled());

        let error = repository
            .get_by_id("application-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn create_returns_disabled_without_database() {
        let repository = ApplicationsRepository::new(Database::disabled());

        let error = repository
            .create(&CreateApplication {
                job_id: "job-1".to_string(),
                status: "saved".to_string(),
                applied_at: None,
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
