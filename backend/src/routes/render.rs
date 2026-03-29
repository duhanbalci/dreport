use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::post,
    Json,
};
use dreport_layout::FontData;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::Template;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub template: Template,
    pub data: serde_json::Value,
}

/// POST /api/render — Template + Data → PDF
pub async fn render(
    State(fonts): State<Arc<Vec<FontData>>>,
    Json(payload): Json<RenderRequest>,
) -> impl IntoResponse {
    // 1. Layout hesapla
    let layout = dreport_layout::compute_layout(&payload.template, &payload.data, &fonts);

    // 2. PDF render
    match dreport_layout::pdf_render::render_pdf(&layout, &fonts) {
        Ok(pdf_bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("PDF render hatasi: {}", err),
        )
            .into_response(),
    }
}

pub fn router() -> Router<Arc<Vec<FontData>>> {
    Router::new().route("/api/render", post(render))
}
