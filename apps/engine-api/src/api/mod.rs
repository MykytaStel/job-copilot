pub mod dto;
pub mod error;
pub mod middleware;
pub mod routes;

use crate::state::AppState;
use axum::Router;
use axum::http::{HeaderValue, Method, Request};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::trace::TraceLayer;
use tracing::warn;

#[derive(Clone)]
struct MakeUuidV7;
pub struct BuiltRouters {
    pub app: Router,
    pub internal: Router,
}

impl MakeRequestId for MakeUuidV7 {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let uuid = uuid::Uuid::now_v7().to_string();
        HeaderValue::from_str(&uuid).ok().map(RequestId::new)
    }
}

pub fn build_routers(state: AppState) -> BuiltRouters {
    let x_request_id = axum::http::header::HeaderName::from_static("x-request-id");
    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    if state.jwt_secret.is_none() {
        warn!("JWT_SECRET is not set; all /api/v1/ routes are unauthenticated");
    }

    let public = routes::public_router();

    let protected = routes::protected_router().route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(build_cors_layer(&state))
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
        .with_state(state.clone());

    let internal = routes::internal_router()
        .route(
            "/metrics",
            axum::routing::get(move || async move { metrics_handle.render() }),
        )
        .with_state(state);

    BuiltRouters { app, internal }
}

fn build_cors_layer(state: &AppState) -> CorsLayer {
    let origins = state
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
}
