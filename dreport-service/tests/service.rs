//! Integration tests for `DreportService`.
//!
//! These tests exercise the public API as it would be consumed by the Axum
//! adapter, the FFI layer, and any other host. Anything that breaks here
//! breaks behaviour for every consumer simultaneously, so failures should
//! be treated as a contract change.

use dreport_service::{DreportService, ServiceError};
use std::sync::Arc;
use std::thread;

const VALID_TEMPLATE: &str = r#"{
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
        "content": "Hello dreport"
      }
    ]
  }
}"#;

const VALID_DATA: &str = r#"{}"#;

const NOTO_SANS_REGULAR: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");

// ---------------------------------------------------------------------------
// Service initialization
// ---------------------------------------------------------------------------

#[test]
fn new_loads_embedded_fonts() {
    let svc = DreportService::new();
    assert!(
        svc.font_family_count() >= 1,
        "embedded-fonts feature should provide at least one family"
    );
    let names: Vec<String> = svc
        .list_font_families()
        .into_iter()
        .map(|f| f.family.to_lowercase())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("noto")),
        "Noto Sans family expected, got {:?}",
        names
    );
}

#[test]
fn empty_starts_with_no_fonts() {
    let svc = DreportService::empty();
    assert_eq!(svc.font_family_count(), 0);
    assert!(svc.list_font_families().is_empty());
}

// ---------------------------------------------------------------------------
// Font registration
// ---------------------------------------------------------------------------

#[test]
fn register_font_bytes_valid_ttf() {
    let svc = DreportService::empty();
    let registered = svc
        .register_font_bytes(NOTO_SANS_REGULAR.to_vec())
        .expect("valid TTF should register");
    assert!(registered.family.to_lowercase().contains("noto"));
    assert_eq!(svc.font_family_count(), 1);
}

#[test]
fn register_font_bytes_invalid_returns_parse_error() {
    let svc = DreportService::empty();
    let err = svc
        .register_font_bytes(b"not a font".to_vec())
        .expect_err("garbage bytes must not parse");
    assert!(matches!(err, ServiceError::FontParseFailed));
    assert_eq!(err.code(), 3);
}

#[test]
fn register_fonts_directory_loads_files() {
    let svc = DreportService::empty();
    let fonts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let count = svc
        .register_fonts_directory(&fonts_dir)
        .expect("assets/fonts must be readable");
    assert!(count >= 1, "at least one font expected in assets/fonts");
    assert!(svc.font_family_count() >= 1);
}

#[test]
fn register_fonts_directory_missing_returns_error() {
    let svc = DreportService::empty();
    let err = svc
        .register_fonts_directory("/no/such/dreport/fonts/path/zzz")
        .expect_err("missing directory must error");
    assert!(matches!(err, ServiceError::FontDirNotFound(_)));
}

#[test]
fn register_fonts_directory_skips_non_font_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("readme.txt"), b"hi").unwrap();
    std::fs::write(dir.path().join("font.ttf"), NOTO_SANS_REGULAR).unwrap();

    let svc = DreportService::empty();
    let count = svc.register_fonts_directory(dir.path()).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn register_fonts_directory_skips_invalid_font_silently() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.ttf"), b"not a font").unwrap();
    std::fs::write(dir.path().join("good.ttf"), NOTO_SANS_REGULAR).unwrap();

    let svc = DreportService::empty();
    let count = svc.register_fonts_directory(dir.path()).unwrap();
    assert_eq!(count, 1, "only the good font should register");
}

// ---------------------------------------------------------------------------
// Font lookup
// ---------------------------------------------------------------------------

#[test]
fn get_font_bytes_returns_data_for_known_variant() {
    let svc = DreportService::new();
    let bytes = svc
        .get_font_bytes("Noto Sans", 400, false)
        .expect("regular variant should exist");
    assert!(!bytes.is_empty());
}

#[test]
fn get_font_bytes_case_insensitive() {
    let svc = DreportService::new();
    let lower = svc.get_font_bytes("noto sans", 400, false);
    let mixed = svc.get_font_bytes("NoTo SaNs", 400, false);
    assert!(lower.is_some());
    assert!(mixed.is_some());
}

#[test]
fn get_font_bytes_unknown_returns_none() {
    let svc = DreportService::new();
    assert!(svc.get_font_bytes("DoesNotExist", 400, false).is_none());
    assert!(svc.get_font_bytes("Noto Sans", 1234, false).is_none());
}

// ---------------------------------------------------------------------------
// Layout + render pipeline
// ---------------------------------------------------------------------------

#[test]
fn compute_layout_json_valid_template_returns_pages() {
    let svc = DreportService::new();
    let json = svc
        .compute_layout_json(VALID_TEMPLATE, VALID_DATA)
        .expect("layout should compute");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let pages = parsed
        .get("pages")
        .and_then(|p| p.as_array())
        .expect("LayoutResult must contain pages array");
    assert!(!pages.is_empty(), "at least one page expected");
}

#[test]
fn compute_layout_json_invalid_template_returns_typed_error() {
    let svc = DreportService::new();
    let err = svc
        .compute_layout_json("{not json", VALID_DATA)
        .expect_err("malformed template must error");
    assert!(matches!(err, ServiceError::InvalidTemplateJson(_)));
    assert_eq!(err.code(), 1);
}

#[test]
fn compute_layout_json_invalid_data_returns_typed_error() {
    let svc = DreportService::new();
    let err = svc
        .compute_layout_json(VALID_TEMPLATE, "{not json")
        .expect_err("malformed data must error");
    assert!(matches!(err, ServiceError::InvalidDataJson(_)));
    assert_eq!(err.code(), 2);
}

#[test]
fn render_pdf_json_produces_pdf_with_magic_header() {
    let svc = DreportService::new();
    let pdf = svc
        .render_pdf_json(VALID_TEMPLATE, VALID_DATA)
        .expect("render must succeed");
    assert!(
        pdf.starts_with(b"%PDF-"),
        "PDF magic header missing; got {:?}",
        &pdf[..pdf.len().min(8)]
    );
    assert!(pdf.len() > 100, "PDF unexpectedly small");
}

#[test]
fn render_pdf_typed_matches_render_pdf_json() {
    let svc = DreportService::new();
    let from_json = svc
        .render_pdf_json(VALID_TEMPLATE, VALID_DATA)
        .expect("json render");
    let template = serde_json::from_str(VALID_TEMPLATE).unwrap();
    let data = serde_json::from_str(VALID_DATA).unwrap();
    let from_typed = svc.render_pdf(&template, &data).expect("typed render");
    // Producer headers vary on time; magic header + non-trivial size sufficient.
    assert!(from_json.starts_with(b"%PDF-"));
    assert!(from_typed.starts_with(b"%PDF-"));
    assert_eq!(from_json.len(), from_typed.len());
}

// ---------------------------------------------------------------------------
// Concurrency
// ---------------------------------------------------------------------------

#[test]
fn concurrent_renders_share_service_safely() {
    let svc = Arc::new(DreportService::new());
    let mut handles = Vec::new();
    for _ in 0..8 {
        let s = Arc::clone(&svc);
        handles.push(thread::spawn(move || {
            let pdf = s.render_pdf_json(VALID_TEMPLATE, VALID_DATA).unwrap();
            assert!(pdf.starts_with(b"%PDF-"));
        }));
    }
    for h in handles {
        h.join().expect("worker panic");
    }
}

#[test]
fn concurrent_register_and_render() {
    let svc = Arc::new(DreportService::new());
    let mut handles = Vec::new();

    let writer_svc = Arc::clone(&svc);
    handles.push(thread::spawn(move || {
        for _ in 0..4 {
            let _ = writer_svc.register_font_bytes(NOTO_SANS_REGULAR.to_vec());
        }
    }));

    for _ in 0..4 {
        let s = Arc::clone(&svc);
        handles.push(thread::spawn(move || {
            let pdf = s.render_pdf_json(VALID_TEMPLATE, VALID_DATA).unwrap();
            assert!(pdf.starts_with(b"%PDF-"));
        }));
    }

    for h in handles {
        h.join().expect("worker panic");
    }
}

// ---------------------------------------------------------------------------
// Error display
// ---------------------------------------------------------------------------

#[test]
fn service_error_codes_are_stable() {
    // FFI consumers depend on these — changing them is a breaking change.
    assert_eq!(ServiceError::InvalidTemplateJson("x".into()).code(), 1);
    assert_eq!(ServiceError::InvalidDataJson("x".into()).code(), 2);
    assert_eq!(ServiceError::FontParseFailed.code(), 3);
    assert_eq!(ServiceError::FontDirNotFound("x".into()).code(), 4);
    assert_eq!(ServiceError::FontDirRead("x".into()).code(), 5);
    assert_eq!(ServiceError::LayoutFailed("x".into()).code(), 6);
    assert_eq!(ServiceError::PdfFailed("x".into()).code(), 7);
    assert_eq!(ServiceError::SerializationFailed("x".into()).code(), 8);
}
