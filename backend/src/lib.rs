//! dreport-backend
//!
//! Thin Axum HTTP adapter on top of `dreport-service`. The HTTP layer holds
//! no business logic — it only translates JSON requests into service calls
//! and maps `ServiceError` into HTTP status codes.

pub mod app;
mod routes;

use axum::Router;
use dreport_service::DreportService;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub use routes::AppState;

/// Build the full Axum `Router` with CORS, state and all `/api/*` endpoints.
pub fn build_router(service: Arc<DreportService>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(routes::router())
        .layer(cors)
        .with_state(service)
}
