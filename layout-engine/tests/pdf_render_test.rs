//! PDF render integration tests.
//! Only compiled on non-WASM targets since pdf_render uses krilla (native only).

#![cfg(not(target_arch = "wasm32"))]

use dreport_core::models::*;
use dreport_layout::compute_layout;

mod common;
use common::load_test_fonts;

fn simple_template() -> Template {
    Template {
        id: "pdf_test".to_string(),
        name: "PDF Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        header: None,
        footer: None,
        format_config: None,
        locale: None,
        root: ContainerElement {
            id: "root".to_string(),
            condition: None,
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
            break_inside: "auto".to_string(),
            children: vec![TemplateElement::StaticText(StaticTextElement {
                id: "title".to_string(),
                condition: None,
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

    let layout = compute_layout(&template, &data, &fonts).unwrap();
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();

    // PDF should not be empty
    assert!(!pdf_bytes.is_empty(), "PDF output should not be empty");

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
        header: None,
        footer: None,
        format_config: None,
        locale: None,
        root: ContainerElement {
            id: "root".to_string(),
            condition: None,
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
            break_inside: "auto".to_string(),
            children: vec![
                TemplateElement::StaticText(StaticTextElement {
                    id: "header".to_string(),
                    condition: None,
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
                    condition: None,
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
                    condition: None,
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

    let layout = compute_layout(&template, &data, &fonts).unwrap();
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
        header: None,
        footer: None,
        format_config: None,
        locale: None,
        root: ContainerElement {
            id: "root".to_string(),
            condition: None,
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
            break_inside: "auto".to_string(),
            children: vec![TemplateElement::StaticText(StaticTextElement {
                id: "text".to_string(),
                condition: None,
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

    let layout = compute_layout(&template, &data, &fonts).unwrap();
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();

    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF"));
}

#[test]
fn test_page_break_produces_multiple_pages() {
    let template = Template {
        id: "pb_test".to_string(),
        name: "Page Break Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        header: None,
        footer: None,
        format_config: None,
        locale: None,
        root: ContainerElement {
            id: "root".to_string(),
            condition: None,
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
            break_inside: "auto".to_string(),
            children: vec![
                TemplateElement::StaticText(StaticTextElement {
                    id: "t1".to_string(),
                    condition: None,
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(18.0),
                        ..Default::default()
                    },
                    content: "Page 1 content".to_string(),
                }),
                TemplateElement::PageBreak(PageBreakElement {
                    id: "pb1".to_string(),
                    condition: None,
                }),
                TemplateElement::StaticText(StaticTextElement {
                    id: "t2".to_string(),
                    condition: None,
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(18.0),
                        ..Default::default()
                    },
                    content: "Page 2 content".to_string(),
                }),
            ],
        },
    };

    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let layout = compute_layout(&template, &data, &fonts).unwrap();

    println!("Layout pages: {}", layout.pages.len());
    for (i, page) in layout.pages.iter().enumerate() {
        println!("Page {}: {} elements", i, page.elements.len());
        for el in &page.elements {
            println!(
                "  - {} (type={}, y={:.1}mm, h={:.1}mm)",
                el.id, el.element_type, el.y_mm, el.height_mm
            );
        }
    }

    assert_eq!(layout.pages.len(), 2, "Page break should produce 2 pages");

    // Verify page 1 has t1 and page 2 has t2
    let p1_ids: Vec<&str> = layout.pages[0]
        .elements
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    let p2_ids: Vec<&str> = layout.pages[1]
        .elements
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    println!("Page 1 IDs: {:?}", p1_ids);
    println!("Page 2 IDs: {:?}", p2_ids);

    assert!(p1_ids.contains(&"t1"), "Page 1 should contain t1");
    assert!(p2_ids.contains(&"t2"), "Page 2 should contain t2");

    // Render PDF and verify it's valid
    let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF"));

    // Write PDF to temp dir for manual inspection
    let out_path = std::env::temp_dir().join("dreport_test_page_break.pdf");
    std::fs::write(&out_path, &pdf_bytes).unwrap();
    println!("Wrote: {}", out_path.display());
}
