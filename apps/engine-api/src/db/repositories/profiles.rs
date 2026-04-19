use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::profile::model::{CreateProfile, Profile, ProfileAnalysis, UpdateProfile};
use crate::domain::role::RoleId;

#[derive(Clone)]
pub struct ProfilesRepository {
    database: Database,
}

#[derive(FromRow)]
struct ProfileRow {
    id: String,
    name: String,
    email: String,
    location: Option<String>,
    raw_text: String,
    summary: Option<String>,
    primary_role: Option<String>,
    seniority: Option<String>,
    skills_json: String,
    keywords_json: String,
    salary_min_usd: Option<i32>,
    salary_max_usd: Option<i32>,
    preferred_work_mode: Option<String>,
    created_at: String,
    updated_at: String,
    skills_updated_at: Option<String>,
}

impl ProfilesRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, input: &CreateProfile) -> Result<Profile, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            INSERT INTO profiles (
                id,
                name,
                email,
                location,
                raw_text,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING
                id,
                name,
                email,
                location,
                raw_text,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min_usd,
                salary_max_usd,
                preferred_work_mode,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&input.name)
        .bind(&input.email)
        .bind(&input.location)
        .bind(&input.raw_text)
        .fetch_one(pool)
        .await?;

        Profile::try_from(row)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Profile>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            SELECT
                id,
                name,
                email,
                location,
                raw_text,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min_usd,
                salary_max_usd,
                preferred_work_mode,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            FROM profiles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.map(Profile::try_from).transpose()
    }

    pub async fn get_latest(&self) -> Result<Option<Profile>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            SELECT
                id,
                name,
                email,
                location,
                raw_text,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min_usd,
                salary_max_usd,
                preferred_work_mode,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            FROM profiles
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        row.map(Profile::try_from).transpose()
    }

    pub async fn update(
        &self,
        id: &str,
        input: &UpdateProfile,
    ) -> Result<Option<Profile>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            UPDATE profiles
            SET
                name = COALESCE($2, name),
                email = COALESCE($3, email),
                location = CASE
                    WHEN $4 THEN $5
                    ELSE location
                END,
                raw_text = COALESCE($6, raw_text),
                summary = CASE
                    WHEN $6 IS NULL THEN summary
                    ELSE NULL
                END,
                primary_role = CASE
                    WHEN $6 IS NULL THEN primary_role
                    ELSE NULL
                END,
                seniority = CASE
                    WHEN $6 IS NULL THEN seniority
                    ELSE NULL
                END,
                skills = CASE
                    WHEN $6 IS NULL THEN skills
                    ELSE '[]'::jsonb
                END,
                keywords = CASE
                    WHEN $6 IS NULL THEN keywords
                    ELSE '[]'::jsonb
                END,
                skills_updated_at = CASE
                    WHEN $6 IS NULL THEN skills_updated_at
                    ELSE NULL
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                name,
                email,
                location,
                raw_text,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min_usd,
                salary_max_usd,
                preferred_work_mode,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            "#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(input.location.is_some())
        .bind(&input.location)
        .bind(&input.raw_text)
        .fetch_optional(pool)
        .await?;

        row.map(Profile::try_from).transpose()
    }

    pub async fn save_analysis(
        &self,
        id: &str,
        analysis: &ProfileAnalysis,
    ) -> Result<Option<Profile>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            UPDATE profiles
            SET
                summary = $2,
                primary_role = $3,
                seniority = $4,
                skills = $5,
                keywords = $6,
                skills_updated_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                name,
                email,
                location,
                raw_text,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min_usd,
                salary_max_usd,
                preferred_work_mode,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            "#,
        )
        .bind(id)
        .bind(&analysis.summary)
        .bind(analysis.primary_role.to_string())
        .bind(&analysis.seniority)
        .bind(Json(analysis.skills.clone()))
        .bind(Json(analysis.keywords.clone()))
        .fetch_optional(pool)
        .await?;

        row.map(Profile::try_from).transpose()
    }
}

impl TryFrom<ProfileRow> for Profile {
    type Error = RepositoryError;

    fn try_from(row: ProfileRow) -> Result<Self, Self::Error> {
        let analysis = match (row.summary, row.primary_role, row.seniority) {
            (Some(summary), Some(primary_role), Some(seniority)) => Some(ProfileAnalysis {
                summary,
                primary_role: RoleId::parse_compat_key(&primary_role).ok_or_else(|| {
                    RepositoryError::InvalidData {
                        message: format!("unknown primary_role stored in profiles: {primary_role}"),
                    }
                })?,
                seniority,
                skills: serde_json::from_str(&row.skills_json)?,
                keywords: serde_json::from_str(&row.keywords_json)?,
            }),
            _ => None,
        };

        Ok(Self {
            id: row.id,
            name: row.name,
            email: row.email,
            location: row.location,
            raw_text: row.raw_text,
            analysis,
            salary_min_usd: row.salary_min_usd,
            salary_max_usd: row.salary_max_usd,
            preferred_work_mode: row.preferred_work_mode,
            created_at: row.created_at,
            updated_at: row.updated_at,
            skills_updated_at: row.skills_updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{ProfilesRepository, RepositoryError};
    use crate::domain::profile::model::CreateProfile;

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = ProfilesRepository::new(Database::disabled());

        let error = repository
            .get_by_id("profile-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn create_returns_disabled_without_database() {
        let repository = ProfilesRepository::new(Database::disabled());

        let error = repository
            .create(&CreateProfile {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: None,
                raw_text: "Senior frontend engineer".to_string(),
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
