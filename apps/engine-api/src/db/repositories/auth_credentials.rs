use sqlx::FromRow;

use crate::db::Database;
use crate::db::repositories::RepositoryError;

#[derive(Clone)]
pub struct AuthCredentialsRepository {
    database: Database,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthCredential {
    pub profile_id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(FromRow)]
struct AuthCredentialRow {
    profile_id: String,
    email: String,
    password_hash: String,
}

impl AuthCredentialsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(
        &self,
        profile_id: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<AuthCredential, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            r#"
            INSERT INTO profile_auth_credentials (
                profile_id,
                email,
                password_hash,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING profile_id, email, password_hash
            "#,
        )
        .bind(profile_id)
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await
        .map_err(map_unique_violation)?;

        Ok(row.into())
    }

    pub async fn get_by_email(
        &self,
        email: &str,
    ) -> Result<Option<AuthCredential>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, AuthCredentialRow>(
            r#"
            SELECT profile_id, email, password_hash
            FROM profile_auth_credentials
            WHERE email = $1
            LIMIT 1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Into::into))
    }
}

fn map_unique_violation(error: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.constraint() == Some("profile_auth_credentials_email_key")
    {
        return RepositoryError::Conflict {
            message: "auth credentials already exist for this email".to_string(),
        };
    }

    RepositoryError::Sqlx(error)
}

impl From<AuthCredentialRow> for AuthCredential {
    fn from(row: AuthCredentialRow) -> Self {
        Self {
            profile_id: row.profile_id,
            email: row.email,
            password_hash: row.password_hash,
        }
    }
}
