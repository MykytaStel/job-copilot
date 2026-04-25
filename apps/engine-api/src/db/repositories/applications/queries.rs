use uuid::Uuid;

use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{
    Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateApplication, CreateApplicationContact, CreateContact, CreateNote, Offer,
    UpdateApplication, UpsertOffer,
};
use crate::domain::search::global::ApplicationSearchHit;

use super::ApplicationsRepository;
use super::rows::{
    ApplicationDetailRow, ApplicationRow, ApplicationSearchHitRow, ContactJoinRow, ContactRow,
    NoteRow, OfferRow,
};

impl ApplicationsRepository {
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
                profile_id,
                job_id,
                resume_id,
                status,
                applied_at,
                due_date,
                updated_at
            )
            VALUES ($1, $2, $3, NULL, $4, $5::timestamptz, NULL, NOW())
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                outcome,
                outcome_date::text AS outcome_date,
                rejection_stage,
                updated_at::text AS updated_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&application.profile_id)
        .bind(&application.job_id)
        .bind(&application.status)
        .bind(&application.applied_at)
        .fetch_one(pool)
        .await
        .map_err(|error| map_create_application_error(error, application))?;

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
                outcome,
                outcome_date::text AS outcome_date,
                rejection_stage,
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
                applications.outcome AS application_outcome,
                applications.outcome_date::text AS application_outcome_date,
                applications.rejection_stage AS application_rejection_stage,
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
                resumes.uploaded_at::text AS resume_uploaded_at,
                notes_agg.data AS notes_json,
                contacts_agg.data AS contacts_json,
                activities_agg.data AS activities_json,
                tasks_agg.data AS tasks_json
            FROM applications
            INNER JOIN jobs ON jobs.id = applications.job_id
            LEFT JOIN resumes ON resumes.id = applications.resume_id
            LEFT JOIN LATERAL (
                SELECT COALESCE(
                    json_agg(
                        json_build_object(
                            'id', id,
                            'application_id', application_id,
                            'content', content,
                            'created_at', created_at::text
                        )
                        ORDER BY created_at DESC
                    ),
                    '[]'::json
                ) AS data
                FROM application_notes
                WHERE application_id = applications.id
            ) notes_agg ON true
            LEFT JOIN LATERAL (
                SELECT COALESCE(
                    json_agg(
                        json_build_object(
                            'id', ac.id,
                            'application_id', ac.application_id,
                            'relationship', ac.relationship,
                            'contact_id', c.id,
                            'contact_name', c.name,
                            'contact_email', c.email,
                            'contact_phone', c.phone,
                            'contact_linkedin_url', c.linkedin_url,
                            'contact_company', c.company,
                            'contact_role', c.role,
                            'contact_created_at', c.created_at::text
                        )
                    ),
                    '[]'::json
                ) AS data
                FROM application_contacts ac
                JOIN contacts c ON c.id = ac.contact_id
                WHERE ac.application_id = applications.id
            ) contacts_agg ON true
            LEFT JOIN LATERAL (
                SELECT COALESCE(
                    json_agg(
                        json_build_object(
                            'id', id,
                            'application_id', application_id,
                            'activity_type', activity_type,
                            'description', description,
                            'happened_at', happened_at::text,
                            'created_at', created_at::text
                        )
                        ORDER BY happened_at DESC
                    ),
                    '[]'::json
                ) AS data
                FROM activities
                WHERE application_id = applications.id
            ) activities_agg ON true
            LEFT JOIN LATERAL (
                SELECT COALESCE(
                    json_agg(
                        json_build_object(
                            'id', id,
                            'application_id', application_id,
                            'title', title,
                            'remind_at', remind_at::text,
                            'done', done,
                            'created_at', created_at::text
                        )
                        ORDER BY created_at ASC
                    ),
                    '[]'::json
                ) AS data
                FROM tasks
                WHERE application_id = applications.id
            ) tasks_agg ON true
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

        Ok(Some(ApplicationDetail::try_from((row, offer))?))
    }

    pub async fn list_recent(
        &self,
        limit: i64,
        profile_id: Option<&str>,
    ) -> Result<Vec<Application>, RepositoryError> {
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
                outcome,
                outcome_date::text AS outcome_date,
                rejection_stage,
                updated_at::text AS updated_at
            FROM applications
            WHERE ($2::text IS NULL OR profile_id = $2)
            ORDER BY updated_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .bind(profile_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Application::from).collect())
    }

    pub async fn search_by_job_title(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ApplicationSearchHit>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ApplicationSearchHitRow>(
            r#"
            SELECT
                applications.id,
                applications.job_id,
                applications.resume_id,
                applications.status,
                applications.applied_at::text AS applied_at,
                applications.due_date::text AS due_date,
                applications.updated_at::text AS updated_at,
                jobs.title AS job_title,
                jobs.company_name
            FROM applications
            INNER JOIN jobs ON jobs.id = applications.job_id
            WHERE to_tsvector('simple', coalesce(jobs.title, '')) @@ plainto_tsquery('simple', $1)
            ORDER BY
                ts_rank_cd(
                    to_tsvector('simple', coalesce(jobs.title, '')),
                    plainto_tsquery('simple', $1)
                ) DESC,
                applications.updated_at DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ApplicationSearchHit::from).collect())
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
                outcome = CASE
                    WHEN $5 THEN $6
                    ELSE outcome
                END,
                outcome_date = CASE
                    WHEN $7 THEN $8::timestamptz
                    ELSE outcome_date
                END,
                rejection_stage = CASE
                    WHEN $9 THEN $10
                    ELSE rejection_stage
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
                outcome,
                outcome_date::text AS outcome_date,
                rejection_stage,
                updated_at::text AS updated_at
            "#,
        )
        .bind(id)
        .bind(&update.status)
        .bind(update.due_date.is_some())
        .bind(update.due_date.as_ref().and_then(|value| value.as_deref()))
        .bind(update.outcome.is_some())
        .bind(
            update
                .outcome
                .as_ref()
                .and_then(|value| value.as_ref().map(|outcome| outcome.as_str())),
        )
        .bind(update.outcome_date.is_some())
        .bind(
            update
                .outcome_date
                .as_ref()
                .and_then(|value| value.as_deref()),
        )
        .bind(update.rejection_stage.is_some())
        .bind(
            update
                .rejection_stage
                .as_ref()
                .and_then(|value| value.as_deref()),
        )
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

    pub async fn list_contacts(&self, offset: i64) -> Result<(Vec<Contact>, i64), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
            .fetch_one(pool)
            .await?;

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
            LIMIT 100 OFFSET $1
            "#,
        )
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((rows.into_iter().map(Contact::from).collect(), total))
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
                outcome,
                outcome_date::text AS outcome_date,
                rejection_stage,
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

fn map_create_application_error(
    error: sqlx::Error,
    application: &CreateApplication,
) -> RepositoryError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.code().as_deref() == Some("23505")
    {
        let scope = application
            .profile_id
            .as_deref()
            .unwrap_or("global application scope");
        return RepositoryError::Conflict {
            message: format!(
                "application already exists for scope '{scope}' and job '{}'",
                application.job_id
            ),
        };
    }
    RepositoryError::Sqlx(error)
}
