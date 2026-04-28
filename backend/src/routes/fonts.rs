use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use super::AppState;

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
async fn list_fonts(State(service): State<AppState>) -> Json<Vec<FontFamilyResponse>> {
    let response: Vec<FontFamilyResponse> = service
        .list_font_families()
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
    State(service): State<AppState>,
    Path((family, weight, italic)): Path<(String, u16, String)>,
) -> impl IntoResponse {
    let is_italic = italic == "true" || italic == "1";

    match service.get_font_bytes(&family, weight, is_italic) {
        Some(data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "font/ttf")],
            data,
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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/fonts", get(list_fonts))
        .route("/api/fonts/{family}/{weight}/{italic}", get(get_font))
}
