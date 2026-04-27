use axum::extract::{Extension, Path, Query, State};
use serde::Deserialize;

use crate::api::dto::notifications::{
    NotificationResponse, NotificationsResponse, UnreadNotificationsCountResponse,
};
use crate::api::error::ApiError;
use crate::api::middleware::auth::AuthUser;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::state::AppState;

#[derive(Debug, Default, Deserialize)]
pub struct NotificationsQuery {
    pub limit: Option<i64>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<NotificationsQuery>,
) -> Result<axum::Json<NotificationsResponse>, ApiError> {
    let profile_id = require_profile_id(auth.as_deref())?;
    ensure_profile_exists(&state, None, &profile_id).await?;

    let limit = query.limit.unwrap_or(20);
    if !(1..=100).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let notifications = state
        .notifications_service
        .list_by_profile(&profile_id, limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "notifications_query_failed"))?;

    Ok(axum::Json(NotificationsResponse {
        notifications: notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect(),
    }))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
) -> Result<axum::Json<NotificationResponse>, ApiError> {
    let Some(notification) = state
        .notifications_service
        .mark_read(&notification_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "notifications_write_failed"))?
    else {
        return Err(ApiError::not_found(
            "notification_not_found",
            format!("Notification '{notification_id}' was not found"),
        ));
    };

    Ok(axum::Json(NotificationResponse::from(notification)))
}

pub async fn get_unread_count(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
) -> Result<axum::Json<UnreadNotificationsCountResponse>, ApiError> {
    let profile_id = require_profile_id(auth.as_deref())?;
    ensure_profile_exists(&state, None, &profile_id).await?;

    let unread_count = state
        .notifications_service
        .unread_count(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "notifications_query_failed"))?;

    Ok(axum::Json(UnreadNotificationsCountResponse {
        profile_id,
        unread_count,
    }))
}

fn require_profile_id(auth: Option<&AuthUser>) -> Result<String, ApiError> {
    auth.map(|u| u.profile_id.clone())
        .ok_or_else(|| ApiError::unauthorized("auth_required", "Authentication is required"))
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::{Extension, Path, Query, State};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::{NotificationsQuery, get_unread_count, list_notifications, mark_notification_read};
    use crate::api::middleware::auth::AuthUser;
    use crate::domain::notification::model::{Notification, NotificationType};
    use crate::domain::profile::model::Profile;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::notifications::{NotificationsService, NotificationsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior frontend engineer".to_string(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: None,
            preferred_language: None,
            search_preferences: None,
            created_at: "2026-04-11T00:00:00Z".to_string(),
            updated_at: "2026-04-11T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    fn sample_notification(id: &str, notification_type: NotificationType) -> Notification {
        Notification {
            id: id.to_string(),
            profile_id: "profile-1".to_string(),
            notification_type,
            title: "2 new jobs matched your profile".to_string(),
            body: Some("Djinni and Work.ua have new backend roles.".to_string()),
            payload: Some(json!({ "count": 2 })),
            read_at: None,
            created_at: "2026-04-19T09:00:00Z".to_string(),
        }
    }

    fn test_state(notifications_service: NotificationsService) -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_notifications_service(notifications_service)
    }

    async fn parse_json_response(response: impl IntoResponse) -> Value {
        let response = response.into_response();
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        serde_json::from_slice(&body).expect("response body should be valid json")
    }

    #[tokio::test]
    async fn list_notifications_returns_latest_first() {
        let state = test_state(NotificationsService::for_tests(
            NotificationsServiceStub::default()
                .with_notification(Notification {
                    created_at: "2026-04-19T10:00:00Z".to_string(),
                    title: "A reactivated role is live again".to_string(),
                    ..sample_notification("notification-2", NotificationType::JobReactivated)
                })
                .with_notification(sample_notification(
                    "notification-1",
                    NotificationType::NewJobsFound,
                )),
        ));

        let payload = parse_json_response(
            list_notifications(
                State(state),
                Some(Extension(AuthUser {
                    profile_id: "profile-1".to_string(),
                })),
                Query(NotificationsQuery { limit: Some(20) }),
            )
            .await
            .expect("notifications list should succeed"),
        )
        .await;

        assert_eq!(payload["notifications"][0]["id"], json!("notification-2"));
        assert_eq!(payload["notifications"][1]["id"], json!("notification-1"));
    }

    #[tokio::test]
    async fn mark_notification_read_sets_read_at() {
        let state = test_state(NotificationsService::for_tests(
            NotificationsServiceStub::default().with_notification(sample_notification(
                "notification-1",
                NotificationType::NewJobsFound,
            )),
        ));

        let payload = parse_json_response(
            mark_notification_read(State(state), Path("notification-1".to_string()))
                .await
                .expect("mark read should succeed"),
        )
        .await;

        assert_eq!(payload["id"], json!("notification-1"));
        assert_eq!(payload["read_at"], json!("2026-04-19T00:00:00Z"));
    }

    #[tokio::test]
    async fn unread_count_returns_only_unread_notifications() {
        let state = test_state(NotificationsService::for_tests(
            NotificationsServiceStub::default()
                .with_notification(Notification {
                    read_at: Some("2026-04-18T12:00:00Z".to_string()),
                    ..sample_notification("notification-2", NotificationType::JobReactivated)
                })
                .with_notification(sample_notification(
                    "notification-1",
                    NotificationType::NewJobsFound,
                )),
        ));

        let payload = parse_json_response(
            get_unread_count(
                State(state),
                Some(Extension(AuthUser {
                    profile_id: "profile-1".to_string(),
                })),
            )
            .await
            .expect("unread count should succeed"),
        )
        .await;

        assert_eq!(
            payload,
            json!({ "profile_id": "profile-1", "unread_count": 1 })
        );
    }
}
