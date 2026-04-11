use axum::Json;

use crate::api::dto::roles::RoleCatalogResponse;

pub async fn list_roles() -> Json<RoleCatalogResponse> {
    Json(RoleCatalogResponse::from_catalog())
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::list_roles;

    #[tokio::test]
    async fn returns_role_catalog_for_clients() {
        let response = list_roles().await.into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert!(
            payload["roles"]
                .as_array()
                .expect("roles should be an array")
                .iter()
                .any(|role| role
                    == &json!({
                        "id": "frontend_developer",
                        "display_name": "Frontend Developer",
                        "family": "engineering",
                        "is_fallback": false
                    }))
        );
    }
}
