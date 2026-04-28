use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use dreport_service::{ServiceError, Template};
use serde::Deserialize;

use super::AppState;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub template: Template,
    pub data: serde_json::Value,
}

/// POST /api/render — Template + Data → PDF
pub async fn render(
    State(service): State<AppState>,
    Json(payload): Json<RenderRequest>,
) -> impl IntoResponse {
    // CPU-intensive layout + PDF render'ı blocking thread'de çalıştır
    let result =
        tokio::task::spawn_blocking(move || service.render_pdf(&payload.template, &payload.data))
            .await;

    match result {
        Ok(Ok(pdf_bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        )
            .into_response(),
        Ok(Err(err)) => (status_for(&err), err.to_string()).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task hatası: {}", err),
        )
            .into_response(),
    }
}

fn status_for(err: &ServiceError) -> StatusCode {
    match err {
        ServiceError::InvalidTemplateJson(_) | ServiceError::InvalidDataJson(_) => {
            StatusCode::BAD_REQUEST
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/render", post(render))
}
