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

use crate::models::Template;
use crate::typst_engine::compiler::compile_pdf;
use crate::typst_engine::template_to_typst::{self, RenderMode};
use typst_kit::fonts::Fonts;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub template: Template,
    pub data: serde_json::Value,
}

/// POST /api/render — Template + Data → PDF
pub async fn render(
    State(fonts): State<Arc<Fonts>>,
    Json(payload): Json<RenderRequest>,
) -> impl IntoResponse {
    // 1. Template JSON → Typst markup
    let typst_markup = template_to_typst::template_to_typst(&payload.template, &payload.data, RenderMode::Pdf);

    // 2. Base64 image'ları çıkar
    let files = template_to_typst::extract_image_files(&payload.template);

    // 3. Typst markup → PDF
    match compile_pdf(typst_markup, &fonts, files) {
        Ok(pdf_bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("PDF derleme hatasi: {}", err),
        )
            .into_response(),
    }
}

pub fn router() -> Router<Arc<Fonts>> {
    Router::new().route("/api/render", post(render))
}
