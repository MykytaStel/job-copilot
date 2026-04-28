use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::api::dto::market::{
    MarketCompaniesResponse, MarketCompanyEntryResponse, MarketOverviewResponse,
    MarketRoleDemandEntryResponse, MarketSalaryTrendResponse,
};
use crate::api::error::ApiError;
use crate::domain::market::model::MarketSource;
use crate::state::AppState;

#[derive(Debug, Default, Deserialize)]
pub struct MarketCompaniesQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MarketSalaryQuery {
    pub seniority: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MarketRolesQuery {
    pub period: Option<i64>,
}

fn live_fallback_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-market-data-source", "live-fallback".parse().unwrap());
    headers
}

pub async fn get_market_overview(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let (overview, source) = state
        .jobs_service
        .market_overview()
        .await
        .map_err(|error| ApiError::from_repository(error, "market_query_failed"))?;

    let headers = if source == MarketSource::Live {
        live_fallback_headers()
    } else {
        HeaderMap::new()
    };

    Ok((headers, Json(MarketOverviewResponse::from(overview))))
}

pub async fn get_market_companies(
    State(state): State<AppState>,
    Query(query): Query<MarketCompaniesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);

    if !(1..=100).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let (companies, source) = state
        .jobs_service
        .market_companies(limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "market_query_failed"))?;

    let headers = if source == MarketSource::Live {
        live_fallback_headers()
    } else {
        HeaderMap::new()
    };

    Ok((
        headers,
        Json(MarketCompaniesResponse {
            companies: companies
                .into_iter()
                .map(MarketCompanyEntryResponse::from)
                .collect(),
        }),
    ))
}

pub async fn get_market_salary_trend(
    State(state): State<AppState>,
    Query(query): Query<MarketSalaryQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let seniority = query
        .seniority
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::bad_request_with_details(
                "invalid_seniority",
                "Query parameter 'seniority' is required",
                serde_json::json!({ "field": "seniority" }),
            )
        })?
        .to_lowercase();

    let (trend, source) = state
        .jobs_service
        .market_salary_trend(&seniority)
        .await
        .map_err(|error| ApiError::from_repository(error, "market_query_failed"))?;

    let trend = trend.ok_or_else(|| {
        ApiError::not_found(
            "market_salary_not_found",
            format!("No salary data found for seniority '{seniority}'"),
        )
    })?;

    let headers = if source == MarketSource::Live {
        live_fallback_headers()
    } else {
        HeaderMap::new()
    };

    Ok((headers, Json(MarketSalaryTrendResponse::from(trend))))
}

pub async fn get_market_salary_trends(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let (trends, source) = state
        .jobs_service
        .market_salary_trends()
        .await
        .map_err(|error| ApiError::from_repository(error, "market_query_failed"))?;

    let headers = if source == MarketSource::Live {
        live_fallback_headers()
    } else {
        HeaderMap::new()
    };

    Ok((
        headers,
        Json(
            trends
                .into_iter()
                .map(MarketSalaryTrendResponse::from)
                .collect::<Vec<_>>(),
        ),
    ))
}

pub async fn get_market_role_demand(
    State(state): State<AppState>,
    Query(query): Query<MarketRolesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let period = query.period.unwrap_or(30);

    if !(1..=365).contains(&period) {
        return Err(ApiError::invalid_period(period));
    }

    let (roles, source) = state
        .jobs_service
        .market_role_demand(period as i32)
        .await
        .map_err(|error| ApiError::from_repository(error, "market_query_failed"))?;

    let headers = if source == MarketSource::Live {
        live_fallback_headers()
    } else {
        HeaderMap::new()
    };

    Ok((
        headers,
        Json(
            roles
                .into_iter()
                .map(MarketRoleDemandEntryResponse::from)
                .collect::<Vec<_>>(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::{
        MarketCompaniesQuery, MarketRolesQuery, MarketSalaryQuery, get_market_companies,
        get_market_overview, get_market_role_demand, get_market_salary_trend,
        get_market_salary_trends,
    };
    use crate::domain::market::model::{
        MarketCompanyEntry, MarketOverview, MarketRoleDemandEntry, MarketSalaryTrend,
        MarketTrendDirection,
    };
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    fn test_state(jobs_service: JobsService) -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            jobs_service,
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    async fn parse_json_response(response: impl IntoResponse) -> Value {
        let response = response.into_response();
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        serde_json::from_slice(&body).expect("response body should be valid json")
    }

    #[tokio::test]
    async fn market_overview_returns_jobs_service_aggregates() {
        let state = test_state(JobsService::for_tests(
            JobsServiceStub::default().with_market_overview(MarketOverview {
                new_jobs_this_week: 42,
                active_companies_count: 18,
                active_jobs_count: 64,
                remote_percentage: 62.5,
            }),
        ));

        let payload = parse_json_response(
            get_market_overview(State(state))
                .await
                .expect("market overview should succeed"),
        )
        .await;

        assert_eq!(
            payload,
            json!({
                "new_jobs_this_week": 42,
                "active_companies_count": 18,
                "active_jobs_count": 64,
                "remote_percentage": 62.5
            })
        );
    }

    #[tokio::test]
    async fn market_companies_returns_velocity_for_each_company() {
        let state = test_state(JobsService::for_tests(
            JobsServiceStub::default().with_market_companies(vec![
                MarketCompanyEntry {
                    company_name: "Acme".to_string(),
                    normalized_company_name: "acme".to_string(),
                    active_jobs: 9,
                    this_week: 5,
                    prev_week: 2,
                    sources: vec!["djinni".to_string()],
                    top_role_groups: vec!["Mobile".to_string()],
                    latest_job_ids: vec!["job-1".to_string()],
                    data_quality_flags: Vec::new(),
                },
                MarketCompanyEntry {
                    company_name: "Beta".to_string(),
                    normalized_company_name: "beta".to_string(),
                    active_jobs: 4,
                    this_week: 1,
                    prev_week: 3,
                    sources: vec!["work_ua".to_string()],
                    top_role_groups: Vec::new(),
                    latest_job_ids: vec!["job-2".to_string()],
                    data_quality_flags: Vec::new(),
                },
            ]),
        ));

        let payload = parse_json_response(
            get_market_companies(
                State(state),
                Query(MarketCompaniesQuery { limit: Some(20) }),
            )
            .await
            .expect("market companies should succeed"),
        )
        .await;

        assert_eq!(
            payload["companies"],
            json!([
                {
                    "company_name": "Acme",
                    "normalized_company_name": "acme",
                    "active_jobs": 9,
                    "this_week": 5,
                    "prev_week": 2,
                    "velocity": 3,
                    "sources": ["djinni"],
                    "top_role_groups": ["Mobile"],
                    "latest_job_ids": ["job-1"],
                    "data_quality_flags": []
                },
                {
                    "company_name": "Beta",
                    "normalized_company_name": "beta",
                    "active_jobs": 4,
                    "this_week": 1,
                    "prev_week": 3,
                    "velocity": -2,
                    "sources": ["work_ua"],
                    "top_role_groups": [],
                    "latest_job_ids": ["job-2"],
                    "data_quality_flags": []
                }
            ])
        );
    }

    #[tokio::test]
    async fn market_companies_validates_limit() {
        let state = test_state(JobsService::for_tests(JobsServiceStub::default()));

        let error = match get_market_companies(
            State(state),
            Query(MarketCompaniesQuery { limit: Some(0) }),
        )
        .await
        {
            Err(e) => e,
            Ok(_) => panic!("limit=0 should be rejected"),
        };

        let payload = parse_json_response(error).await;

        assert_eq!(payload["code"], json!("invalid_limit"));
        assert_eq!(payload["details"]["field"], json!("limit"));
        assert_eq!(payload["details"]["received"], json!(0));
    }

    #[tokio::test]
    async fn market_salary_trend_returns_percentiles_for_requested_seniority() {
        let state = test_state(JobsService::for_tests(
            JobsServiceStub::default().with_market_salary_trend(MarketSalaryTrend {
                seniority: "senior".to_string(),
                currency: "USD".to_string(),
                p25: 4200,
                median: 5000,
                p75: 6200,
                sample_count: 8,
            }),
        ));

        let payload = parse_json_response(
            get_market_salary_trend(
                State(state),
                Query(MarketSalaryQuery {
                    seniority: Some("Senior".to_string()),
                }),
            )
            .await
            .expect("market salary trend should succeed"),
        )
        .await;

        assert_eq!(
            payload,
            json!({
                "seniority": "senior",
                "currency": "USD",
                "p25": 4200,
                "median": 5000,
                "p75": 6200,
                "sample_count": 8
            })
        );
    }

    #[tokio::test]
    async fn market_salary_trend_requires_seniority() {
        let state = test_state(JobsService::for_tests(JobsServiceStub::default()));

        let error = match get_market_salary_trend(
            State(state),
            Query(MarketSalaryQuery {
                seniority: Some(" ".to_string()),
            }),
        )
        .await
        {
            Err(error) => error,
            Ok(_) => panic!("blank seniority should be rejected"),
        };

        let payload = parse_json_response(error).await;

        assert_eq!(payload["code"], json!("invalid_seniority"));
        assert_eq!(payload["details"]["field"], json!("seniority"));
    }

    #[tokio::test]
    async fn market_salary_trends_returns_available_buckets() {
        let state = test_state(JobsService::for_tests(
            JobsServiceStub::default()
                .with_market_salary_trend(MarketSalaryTrend {
                    seniority: "senior".to_string(),
                    currency: "USD".to_string(),
                    p25: 4200,
                    median: 5000,
                    p75: 6200,
                    sample_count: 8,
                })
                .with_market_salary_trend(MarketSalaryTrend {
                    seniority: "junior".to_string(),
                    currency: "USD".to_string(),
                    p25: 1200,
                    median: 1500,
                    p75: 1800,
                    sample_count: 6,
                }),
        ));

        let payload = parse_json_response(
            get_market_salary_trends(State(state))
                .await
                .expect("market salary trends should succeed"),
        )
        .await;

        assert_eq!(
            payload,
            json!([
                {
                    "seniority": "junior",
                    "currency": "USD",
                    "p25": 1200,
                    "median": 1500,
                    "p75": 1800,
                    "sample_count": 6
                },
                {
                    "seniority": "senior",
                    "currency": "USD",
                    "p25": 4200,
                    "median": 5000,
                    "p75": 6200,
                    "sample_count": 8
                }
            ])
        );
    }

    #[tokio::test]
    async fn market_role_demand_returns_trend_for_each_group() {
        let state = test_state(JobsService::for_tests(
            JobsServiceStub::default().with_market_role_demand(vec![
                MarketRoleDemandEntry {
                    role_group: "Frontend".to_string(),
                    this_period: 6,
                    prev_period: 3,
                    trend: MarketTrendDirection::Up,
                },
                MarketRoleDemandEntry {
                    role_group: "Backend".to_string(),
                    this_period: 4,
                    prev_period: 4,
                    trend: MarketTrendDirection::Stable,
                },
                MarketRoleDemandEntry {
                    role_group: "QA".to_string(),
                    this_period: 1,
                    prev_period: 2,
                    trend: MarketTrendDirection::Down,
                },
            ]),
        ));

        let payload = parse_json_response(
            get_market_role_demand(State(state), Query(MarketRolesQuery { period: Some(30) }))
                .await
                .expect("market role demand should succeed"),
        )
        .await;

        assert_eq!(
            payload,
            json!([
                {
                    "role_group": "Frontend",
                    "this_period": 6,
                    "prev_period": 3,
                    "trend": "up"
                },
                {
                    "role_group": "Backend",
                    "this_period": 4,
                    "prev_period": 4,
                    "trend": "stable"
                },
                {
                    "role_group": "QA",
                    "this_period": 1,
                    "prev_period": 2,
                    "trend": "down"
                }
            ])
        );
    }

    #[tokio::test]
    async fn market_role_demand_validates_period() {
        let state = test_state(JobsService::for_tests(JobsServiceStub::default()));

        let error =
            match get_market_role_demand(State(state), Query(MarketRolesQuery { period: Some(0) }))
                .await
            {
                Err(error) => error,
                Ok(_) => panic!("period=0 should be rejected"),
            };

        let payload = parse_json_response(error).await;

        assert_eq!(payload["code"], json!("invalid_period"));
        assert_eq!(payload["details"]["field"], json!("period"));
        assert_eq!(payload["details"]["received"], json!(0));
    }
}
