use axum::{Router, routing::get, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::font_registry::FontRegistry;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub fn router() -> Router<Arc<FontRegistry>> {
    Router::new().route("/api/health", get(health))
}
