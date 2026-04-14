use axum::Json;

use crate::api::dto::sources::SourceCatalogResponse;

pub async fn list_sources() -> Json<SourceCatalogResponse> {
    Json(SourceCatalogResponse::from_catalog())
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::list_sources;

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
                { "id": "work_ua", "display_name": "Work.ua" },
                { "id": "robota_ua", "display_name": "Robota.ua" }
            ])
        );
    }
}
