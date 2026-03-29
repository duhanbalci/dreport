//! PDF render integration tests.
//! Only compiled on non-WASM targets since pdf_render uses krilla (native only).

#![cfg(not(target_arch = "wasm32"))]

use dreport_core::models::*;
use dreport_layout::{compute_layout, FontData};

fn load_test_fonts() -> Vec<FontData> {
    let font_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("backend/fonts");

    let mut fonts = Vec::new();
    for entry in std::fs::read_dir(&font_dir).expect("backend/fonts directory not found") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ttf") {
            let family = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .split('-')
                .next()
                .unwrap_or("Unknown")
                .to_string();
            let family = if family == "NotoSansMono" {
                "Noto Sans Mono".to_string()
            } else if family == "NotoSans" {
                "Noto Sans".to_string()
            } else {
                family
            };
            fonts.push(FontData {
                family,
                data: std::fs::read(&path).unwrap(),
            });
        }
    }
    fonts
}

fn simple_template() -> Template {
    Template {
        id: "pdf_test".to_string(),
        name: "PDF Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        root: ContainerElement {
            id: "root".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 5.0,
            padding: Padding {
                top: 15.0,
                right: 15.0,
                bottom: 15.0,
                left: 15.0,
            },
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle::default(),
            children: vec![TemplateElement::StaticText(StaticTextElement {
                id: "title".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Fr { value: 1.0 },
                    height: SizeValue::Auto,
                    ..Default::default()
                },
                style: TextStyle {
                    font_size: Some(18.0),
                    font_weight: Some("bold".to_string()),
                    ..Default::default()
                },
                content: "PDF Render Test".to_string(),
            })],
        },
    }
}

#[test]
fn test_render_pdf_produces_valid_output() {
    let template = simple_template();
    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let layout = compute_layout(&template, &data, &fonts);
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();

    // PDF should not be empty
    assert!(
        !pdf_bytes.is_empty(),
        "PDF output should not be empty"
    );

    // PDF should start with %PDF magic bytes
    assert!(
        pdf_bytes.starts_with(b"%PDF"),
        "PDF output should start with %PDF magic bytes, got: {:?}",
        &pdf_bytes[..std::cmp::min(10, pdf_bytes.len())]
    );
}

#[test]
fn test_render_pdf_with_multiple_elements() {
    let template = Template {
        id: "pdf_multi".to_string(),
        name: "PDF Multi".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        root: ContainerElement {
            id: "root".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 5.0,
            padding: Padding {
                top: 15.0,
                right: 15.0,
                bottom: 15.0,
                left: 15.0,
            },
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle::default(),
            children: vec![
                TemplateElement::StaticText(StaticTextElement {
                    id: "header".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(16.0),
                        font_weight: Some("bold".to_string()),
                        ..Default::default()
                    },
                    content: "FATURA".to_string(),
                }),
                TemplateElement::Line(LineElement {
                    id: "sep".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: LineStyle {
                        stroke_color: Some("#000000".to_string()),
                        stroke_width: Some(0.5),
                    },
                }),
                TemplateElement::StaticText(StaticTextElement {
                    id: "body".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(11.0),
                        ..Default::default()
                    },
                    content: "Bu bir test belgesidir.".to_string(),
                }),
            ],
        },
    };

    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let layout = compute_layout(&template, &data, &fonts);
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();

    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF"));

    // A PDF with multiple elements should be reasonably sized
    assert!(
        pdf_bytes.len() > 100,
        "PDF with multiple elements should be >100 bytes, got {}",
        pdf_bytes.len()
    );
}

#[test]
fn test_render_pdf_with_container_styles() {
    let template = Template {
        id: "pdf_styled".to_string(),
        name: "PDF Styled".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        root: ContainerElement {
            id: "root".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 0.0,
            padding: Padding {
                top: 20.0,
                right: 20.0,
                bottom: 20.0,
                left: 20.0,
            },
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle {
                background_color: Some("#f0f0f0".to_string()),
                border_color: Some("#333333".to_string()),
                border_width: Some(1.0),
                ..Default::default()
            },
            children: vec![TemplateElement::StaticText(StaticTextElement {
                id: "text".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Fr { value: 1.0 },
                    height: SizeValue::Auto,
                    ..Default::default()
                },
                style: TextStyle {
                    font_size: Some(12.0),
                    color: Some("#ff0000".to_string()),
                    ..Default::default()
                },
                content: "Styled text".to_string(),
            })],
        },
    };

    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let layout = compute_layout(&template, &data, &fonts);
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();

    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF"));
}
