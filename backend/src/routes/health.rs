use axum::{Router, routing::get, Json};
use dreport_layout::FontData;
use serde::Serialize;
use std::sync::Arc;

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

pub fn router() -> Router<Arc<Vec<FontData>>> {
    Router::new().route("/api/health", get(health))
}
