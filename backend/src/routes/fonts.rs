use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use dreport_layout::font_provider::FontProvider;
use serde::Serialize;
use std::sync::Arc;

use crate::font_registry::FontRegistry;

#[derive(Serialize)]
struct FontFamilyResponse {
    family: String,
    variants: Vec<FontVariantResponse>,
}

#[derive(Serialize)]
struct FontVariantResponse {
    weight: u16,
    italic: bool,
}

/// GET /api/fonts — list all available font families
async fn list_fonts(State(registry): State<Arc<FontRegistry>>) -> Json<Vec<FontFamilyResponse>> {
    let families = registry.list_families();
    let response: Vec<FontFamilyResponse> = families
        .into_iter()
        .map(|f| FontFamilyResponse {
            family: f.family,
            variants: f
                .variants
                .into_iter()
                .map(|v| FontVariantResponse {
                    weight: v.weight,
                    italic: v.italic,
                })
                .collect(),
        })
        .collect();
    Json(response)
}

/// GET /api/fonts/:family/:weight/:italic — serve font binary
async fn get_font(
    State(registry): State<Arc<FontRegistry>>,
    Path((family, weight, italic)): Path<(String, u16, String)>,
) -> impl IntoResponse {
    let is_italic = italic == "true" || italic == "1";

    match registry.get_font_bytes(&family, weight, is_italic) {
        Some(data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "font/ttf")],
            data.to_vec(),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!(
                "Font bulunamadı: {} weight={} italic={}",
                family, weight, is_italic
            ),
        )
            .into_response(),
    }
}

pub fn router() -> Router<Arc<FontRegistry>> {
    Router::new()
        .route("/api/fonts", get(list_fonts))
        .route("/api/fonts/{family}/{weight}/{italic}", get(get_font))
}
