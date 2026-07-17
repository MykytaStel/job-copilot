use sqlx::FromRow;

use crate::db::Database;
use crate::domain::profile::onboarding::{OnboardingOutcome, OnboardingStep, ProfileOnboarding};

use super::RepositoryError;

#[derive(Clone)]
pub struct ProfileOnboardingRepository {
    database: Database,
}

#[derive(FromRow)]
struct ProfileOnboardingRow {
    profile_id: String,
    current_step: String,
    completed_steps: Vec<String>,
    skipped_steps: Vec<String>,
    completed_at: Option<String>,
    updated_at: String,
}

const ONBOARDING_SELECT: &str = r#"
    profile_id,
    current_step,
    completed_steps,
    skipped_steps,
    completed_at::text AS completed_at,
    updated_at::text AS updated_at
"#;

impl ProfileOnboardingRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_or_create(
        &self,
        profile_id: &str,
    ) -> Result<ProfileOnboarding, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query(
            r#"
            INSERT INTO profile_onboarding (profile_id)
            VALUES ($1)
            ON CONFLICT (profile_id) DO NOTHING
            "#,
        )
        .bind(profile_id)
        .execute(pool)
        .await?;

        let row = sqlx::query_as::<_, ProfileOnboardingRow>(&format!(
            "SELECT {ONBOARDING_SELECT} FROM profile_onboarding WHERE profile_id = $1"
        ))
        .bind(profile_id)
        .fetch_one(pool)
        .await?;

        row.try_into()
    }

    pub async fn advance(
        &self,
        profile_id: &str,
        step: OnboardingStep,
        outcome: OnboardingOutcome,
    ) -> Result<ProfileOnboarding, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };
        let mut transaction = pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO profile_onboarding (profile_id)
            VALUES ($1)
            ON CONFLICT (profile_id) DO NOTHING
            "#,
        )
        .bind(profile_id)
        .execute(&mut *transaction)
        .await?;

        let row = sqlx::query_as::<_, ProfileOnboardingRow>(&format!(
            "SELECT {ONBOARDING_SELECT} FROM profile_onboarding WHERE profile_id = $1 FOR UPDATE"
        ))
        .bind(profile_id)
        .fetch_one(&mut *transaction)
        .await?;
        let mut state = ProfileOnboarding::try_from(row)?;
        state
            .advance(step, outcome)
            .map_err(|message| RepositoryError::Conflict { message })?;

        let completed_steps = state
            .completed_steps
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>();
        let skipped_steps = state
            .skipped_steps
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>();
        let row = sqlx::query_as::<_, ProfileOnboardingRow>(&format!(
            r#"
            UPDATE profile_onboarding
            SET current_step = $2,
                completed_steps = $3,
                skipped_steps = $4,
                completed_at = CASE WHEN $5 THEN COALESCE(completed_at, NOW()) ELSE completed_at END,
                updated_at = NOW()
            WHERE profile_id = $1
            RETURNING {ONBOARDING_SELECT}
            "#
        ))
        .bind(profile_id)
        .bind(state.current_step.as_str())
        .bind(completed_steps)
        .bind(skipped_steps)
        .bind(state.current_step == OnboardingStep::Complete)
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;
        row.try_into()
    }
}

impl TryFrom<ProfileOnboardingRow> for ProfileOnboarding {
    type Error = RepositoryError;

    fn try_from(row: ProfileOnboardingRow) -> Result<Self, Self::Error> {
        Ok(Self {
            profile_id: row.profile_id,
            current_step: parse_step(&row.current_step)?,
            completed_steps: row
                .completed_steps
                .iter()
                .map(|step| parse_step(step))
                .collect::<Result<Vec<_>, _>>()?,
            skipped_steps: row
                .skipped_steps
                .iter()
                .map(|step| parse_step(step))
                .collect::<Result<Vec<_>, _>>()?,
            completed_at: row.completed_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_step(value: &str) -> Result<OnboardingStep, RepositoryError> {
    OnboardingStep::parse(value).ok_or_else(|| RepositoryError::InvalidData {
        message: format!("unknown onboarding step stored for profile: {value}"),
    })
}
