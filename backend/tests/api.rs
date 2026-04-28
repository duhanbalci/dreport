//! End-to-end HTTP tests for the backend. Drives the real `Router` via
//! `tower::ServiceExt::oneshot`, so anything covered here protects the
//! contract that the editor and external clients rely on.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use dreport_backend::build_router;
use dreport_service::DreportService;
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

const TEMPLATE: &str = r#"{
  "id": "test",
  "name": "Test",
  "page": { "width": 210, "height": 297 },
  "fonts": ["Noto Sans"],
  "root": {
    "id": "root",
    "type": "container",
    "position": { "type": "flow" },
    "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
    "direction": "column",
    "gap": 5,
    "padding": { "top": 15, "right": 15, "bottom": 15, "left": 15 },
    "align": "stretch",
    "justify": "start",
    "style": {},
    "children": [
      {
        "id": "title",
        "type": "static_text",
        "position": { "type": "flow" },
        "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
        "style": { "fontSize": 14, "fontWeight": "bold" },
        "content": "Hello"
      }
    ]
  }
}"#;

fn router() -> axum::Router {
    build_router(Arc::new(DreportService::new()))
}

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body().collect().await.unwrap().to_bytes().to_vec()
}

#[tokio::test]
async fn health_returns_ok() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_bytes(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn list_fonts_includes_noto_sans() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/api/fonts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_bytes(resp).await;
    let families: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        families
            .iter()
            .any(|f| f["family"].as_str().unwrap_or("").to_lowercase().contains("noto")),
        "Noto Sans family should be listed: {:?}",
        families
    );
}

#[tokio::test]
async fn get_font_bytes_for_known_variant() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/api/fonts/Noto%20Sans/400/false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap()),
        Some("font/ttf")
    );
    let body = body_bytes(resp).await;
    assert!(body.len() > 1000, "TTF body should be substantial");
}

#[tokio::test]
async fn get_font_unknown_returns_404() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/api/fonts/DoesNotExist/400/false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn render_returns_pdf_bytes() {
    let payload = serde_json::json!({
        "template": serde_json::from_str::<serde_json::Value>(TEMPLATE).unwrap(),
        "data": {}
    });
    let resp = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/render")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap()),
        Some("application/pdf")
    );
    let body = body_bytes(resp).await;
    assert!(body.starts_with(b"%PDF-"), "PDF magic header missing");
}

#[tokio::test]
async fn render_with_invalid_template_field_returns_4xx_or_500() {
    // Axum's Json extractor rejects malformed payloads with 4xx; a structurally
    // valid but semantically invalid template would surface as 500. Either is
    // acceptable, but the server must not panic and must produce a body.
    let payload = serde_json::json!({ "template": "not an object", "data": {} });
    let resp = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/render")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "got unexpected status {}",
        resp.status()
    );
}
