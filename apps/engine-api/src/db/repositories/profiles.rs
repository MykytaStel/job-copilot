use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::profile::model::{CreateProfile, Profile, ProfileAnalysis, UpdateProfile};
use crate::domain::role::RoleId;
use crate::domain::search::profile::SearchPreferences;

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
    years_of_experience: Option<i32>,
    summary: Option<String>,
    primary_role: Option<String>,
    seniority: Option<String>,
    skills_json: String,
    keywords_json: String,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    languages_json: String,
    preferred_work_mode: Option<String>,
    search_preferences: Option<Json<SearchPreferences>>,
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
                years_of_experience,
                salary_min,
                salary_max,
                salary_currency,
                languages,
                search_preferences,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING
                id,
                name,
                email,
                location,
                raw_text,
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                salary_currency,
                languages::text AS languages_json,
                preferred_work_mode,
                search_preferences,
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
        .bind(input.years_of_experience)
        .bind(input.salary_min)
        .bind(input.salary_max)
        .bind(&input.salary_currency)
        .bind(Json(input.languages.clone()))
        .bind(
            input
                .search_preferences
                .as_ref()
                .map(|value| Json(value.clone())),
        )
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
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                COALESCE(salary_currency, 'USD') AS salary_currency,
                COALESCE(languages, '[]'::jsonb)::text AS languages_json,
                preferred_work_mode,
                search_preferences,
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

    pub async fn get_by_email(&self, email: &str) -> Result<Option<Profile>, RepositoryError> {
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
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                COALESCE(salary_currency, 'USD') AS salary_currency,
                COALESCE(languages, '[]'::jsonb)::text AS languages_json,
                preferred_work_mode,
                search_preferences,
                created_at::text AS created_at,
                updated_at::text AS updated_at,
                skills_updated_at::text AS skills_updated_at
            FROM profiles
            WHERE email = $1
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .bind(email)
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
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                COALESCE(salary_currency, 'USD') AS salary_currency,
                COALESCE(languages, '[]'::jsonb)::text AS languages_json,
                preferred_work_mode,
                search_preferences,
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
                years_of_experience = CASE
                    WHEN $7 THEN $8
                    ELSE years_of_experience
                END,
                salary_min = CASE
                    WHEN $9 THEN $10
                    ELSE salary_min
                END,
                salary_max = CASE
                    WHEN $11 THEN $12
                    ELSE salary_max
                END,
                salary_currency = COALESCE($13, salary_currency, 'USD'),
                languages = CASE
                    WHEN $14 THEN $15
                    ELSE languages
                END,
                search_preferences = CASE
                    WHEN $16 THEN $17
                    ELSE search_preferences
                END,
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
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                COALESCE(salary_currency, 'USD') AS salary_currency,
                COALESCE(languages, '[]'::jsonb)::text AS languages_json,
                preferred_work_mode,
                search_preferences,
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
        .bind(input.years_of_experience.is_some())
        .bind(input.years_of_experience.flatten())
        .bind(input.salary_min.is_some())
        .bind(input.salary_min.flatten())
        .bind(input.salary_max.is_some())
        .bind(input.salary_max.flatten())
        .bind(&input.salary_currency)
        .bind(input.languages.is_some())
        .bind(input.languages.as_ref().map(|value| Json(value.clone())))
        .bind(input.search_preferences.is_some())
        .bind(
            input
                .search_preferences
                .as_ref()
                .and_then(|value| value.as_ref().map(|prefs| Json(prefs.clone()))),
        )
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
                years_of_experience,
                summary,
                primary_role,
                seniority,
                skills::text AS skills_json,
                keywords::text AS keywords_json,
                salary_min,
                salary_max,
                COALESCE(salary_currency, 'USD') AS salary_currency,
                COALESCE(languages, '[]'::jsonb)::text AS languages_json,
                preferred_work_mode,
                search_preferences,
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
            years_of_experience: row.years_of_experience,
            analysis,
            salary_min: row.salary_min,
            salary_max: row.salary_max,
            salary_currency: row.salary_currency.unwrap_or_else(|| "USD".to_string()),
            languages: serde_json::from_str(&row.languages_json)?,
            preferred_work_mode: row.preferred_work_mode,
            search_preferences: row.search_preferences.map(|value| value.0),
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
                years_of_experience: None,
                salary_min: None,
                salary_max: None,
                salary_currency: "USD".to_string(),
                languages: vec![],
                search_preferences: None,
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
