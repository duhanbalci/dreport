use axum::{Router, routing::get, Json};
use serde::Serialize;
use std::sync::Arc;
use typst_kit::fonts::Fonts;

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

pub fn router() -> Router<Arc<Fonts>> {
    Router::new().route("/api/health", get(health))
}
