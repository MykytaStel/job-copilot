use std::collections::BTreeMap;

use axum::{Json, extract::State};
use sqlx::FromRow;

use crate::api::dto::sources::{
    SourceCatalogResponse, SourceHealthItemResponse, SourceHealthResponse, SourceHealthRunResponse,
};
use crate::api::error::ApiError;
use crate::domain::source::SOURCE_CATALOG;
use crate::state::AppState;

pub async fn list_sources() -> Json<SourceCatalogResponse> {
    Json(SourceCatalogResponse::from_catalog())
}

pub async fn get_source_health(
    State(state): State<AppState>,
) -> Result<Json<SourceHealthResponse>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::service_unavailable(
            "database_unavailable",
            "Database is not configured",
        ));
    };

    let runs = sqlx::query_as::<_, SourceHealthRow>(
        r#"
        WITH ranked AS (
            SELECT
                source,
                run_at,
                jobs_fetched,
                jobs_upserted,
                errors,
                duration_ms,
                status,
                ROW_NUMBER() OVER (
                    PARTITION BY source
                    ORDER BY run_at DESC, id DESC
                ) AS run_rank
            FROM ingestion_runs
        ),
        degraded_sources AS (
            SELECT
                source,
                COUNT(*) = 3 AND BOOL_AND(jobs_fetched = 0) AS degraded
            FROM ranked
            WHERE run_rank <= 3
            GROUP BY source
        )
        SELECT
            latest.source,
            latest.run_at::text AS run_at,
            latest.jobs_fetched,
            latest.jobs_upserted,
            latest.errors,
            latest.duration_ms,
            latest.status,
            COALESCE(degraded_sources.degraded, FALSE) AS degraded
        FROM ranked latest
        LEFT JOIN degraded_sources ON degraded_sources.source = latest.source
        WHERE latest.run_rank = 1
        ORDER BY latest.source
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| {
        tracing::error!(
            error = %error,
            "failed to query source health"
        );
        ApiError::internal("source_health_query_failed", error.to_string())
    })?;

    Ok(Json(SourceHealthResponse {
        sources: build_source_health_response(runs)?,
    }))
}

#[derive(Debug, FromRow)]
struct SourceHealthRow {
    source: String,
    run_at: String,
    jobs_fetched: i32,
    jobs_upserted: i32,
    errors: i32,
    duration_ms: i64,
    status: String,
    degraded: bool,
}

fn build_source_health_response(
    runs: Vec<SourceHealthRow>,
) -> Result<Vec<SourceHealthItemResponse>, ApiError> {
    let runs_by_source = runs
        .into_iter()
        .map(|run| (run.source.clone(), run))
        .collect::<BTreeMap<_, _>>();

    SOURCE_CATALOG
        .iter()
        .map(|source| {
            let run = runs_by_source.get(source.canonical_key);
            let status = run
                .map(|run| {
                    if run.degraded {
                        "degraded".to_string()
                    } else {
                        run.status.clone()
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());

            Ok(SourceHealthItemResponse {
                source: source.canonical_key.to_string(),
                display_name: source.display_name.to_string(),
                status,
                degraded: run.map(|run| run.degraded).unwrap_or(false),
                last_run: run.map(source_health_run_response).transpose()?,
            })
        })
        .collect()
}

fn source_health_run_response(run: &SourceHealthRow) -> Result<SourceHealthRunResponse, ApiError> {
    Ok(SourceHealthRunResponse {
        run_at: run.run_at.clone(),
        jobs_fetched: u32::try_from(run.jobs_fetched).map_err(|_| {
            ApiError::internal(
                "source_health_invalid_data",
                "ingestion run jobs_fetched is negative",
            )
        })?,
        jobs_upserted: u32::try_from(run.jobs_upserted).map_err(|_| {
            ApiError::internal(
                "source_health_invalid_data",
                "ingestion run jobs_upserted is negative",
            )
        })?,
        errors: u32::try_from(run.errors).map_err(|_| {
            ApiError::internal(
                "source_health_invalid_data",
                "ingestion run errors is negative",
            )
        })?,
        duration_ms: u64::try_from(run.duration_ms).map_err(|_| {
            ApiError::internal(
                "source_health_invalid_data",
                "ingestion run duration_ms is negative",
            )
        })?,
        status: if run.degraded {
            "degraded".to_string()
        } else {
            run.status.clone()
        },
    })
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::{SourceHealthRow, build_source_health_response, list_sources};

    #[tokio::test]
    async fn returns_source_catalog_for_clients() {
        let response = list_sources().await.into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(
            payload["sources"],
            json!([
                { "id": "djinni", "display_name": "Djinni" },
                { "id": "dou_ua", "display_name": "DOU" },
                { "id": "work_ua", "display_name": "Work.ua" },
                { "id": "robota_ua", "display_name": "Robota.ua" }
            ])
        );
    }

    #[test]
    fn source_health_marks_degraded_sources() {
        let response = build_source_health_response(vec![SourceHealthRow {
            source: "djinni".to_string(),
            run_at: "2026-04-29 18:00:00+00".to_string(),
            jobs_fetched: 0,
            jobs_upserted: 0,
            errors: 0,
            duration_ms: 123,
            status: "ok".to_string(),
            degraded: true,
        }])
        .expect("health response should build");

        let djinni = response
            .iter()
            .find(|source| source.source == "djinni")
            .expect("catalog source should be present");
        let work_ua = response
            .iter()
            .find(|source| source.source == "work_ua")
            .expect("catalog source should be present");

        assert_eq!(djinni.status, "degraded");
        assert!(djinni.degraded);
        assert_eq!(
            djinni.last_run.as_ref().map(|run| run.status.as_str()),
            Some("degraded")
        );
        assert_eq!(work_ua.status, "unknown");
        assert!(!work_ua.degraded);
        assert!(work_ua.last_run.is_none());
    }
}
