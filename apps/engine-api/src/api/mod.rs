pub mod dto;
pub mod error;
pub mod routes;

use axum::Router;
use axum::http::{HeaderValue, Request};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

#[derive(Clone)]
struct MakeUuidV7;

impl MakeRequestId for MakeUuidV7 {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let uuid = uuid::Uuid::now_v7().to_string();
        HeaderValue::from_str(&uuid).ok().map(RequestId::new)
    }
}

pub fn build_router(state: AppState) -> Router {
    let x_request_id = axum::http::header::HeaderName::from_static("x-request-id");
    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    Router::new()
        .merge(routes::router())
        .route(
            "/metrics",
            axum::routing::get(move || async move { metrics_handle.render() }),
        )
        .layer(CorsLayer::permissive())
        .layer(prometheus_layer)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .and_then(|id| id.header_value().to_str().ok())
                    .unwrap_or("-")
                    .to_owned();
                tracing::info_span!(
                    "request",
                    request_id = %request_id,
                    method = %request.method(),
                    uri = %request.uri().path(),
                )
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(x_request_id, MakeUuidV7))
        .with_state(state)
}
