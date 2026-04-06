use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::post,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::font_registry::FontRegistry;
use crate::models::Template;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub template: Template,
    pub data: serde_json::Value,
}

/// POST /api/render — Template + Data → PDF
pub async fn render(
    State(registry): State<Arc<FontRegistry>>,
    Json(payload): Json<RenderRequest>,
) -> impl IntoResponse {
    // CPU-intensive layout + PDF render'ı blocking thread'de çalıştır
    let result = tokio::task::spawn_blocking(move || {
        // Template'in fonts alanına göre sadece gerekli fontları yükle
        let fonts = registry.fonts_for_families(&payload.template.fonts);
        let layout = dreport_layout::compute_layout(&payload.template, &payload.data, &fonts)
            .map_err(|e| format!("Layout error: {}", e))?;
        dreport_layout::pdf_render::render_pdf(&layout, &fonts)
    })
    .await;

    match result {
        Ok(Ok(pdf_bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        )
            .into_response(),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("PDF render hatası: {}", err),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task hatası: {}", err),
        )
            .into_response(),
    }
}

pub fn router() -> Router<Arc<FontRegistry>> {
    Router::new().route("/api/render", post(render))
}
