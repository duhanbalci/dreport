//! Integration tests for the layout engine's compute_layout() public API.

use dreport_core::models::*;
use dreport_layout::{compute_layout, FontData, LayoutResult};

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
            let data = std::fs::read(&path).unwrap();
            if let Some(fd) = FontData::from_bytes(data) {
                fonts.push(fd);
            }
        }
    }
    fonts
}

fn simple_template() -> Template {
    Template {
        id: "test".to_string(),
        name: "Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        header: None,
        footer: None,
        format_config: None,
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
            break_inside: "auto".to_string(),
            children: vec![TemplateElement::StaticText(StaticTextElement {
                id: "title".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Fr { value: 1.0 },
                    height: SizeValue::Auto,
                    ..Default::default()
                },
                style: TextStyle {
                    font_size: Some(14.0),
                    font_weight: Some("bold".to_string()),
                    ..Default::default()
                },
                content: "Hello World".to_string(),
            })],
        },
    }
}

#[test]
fn test_compute_layout_single_page() {
    let template = simple_template();
    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let result: LayoutResult = compute_layout(&template, &data, &fonts).unwrap();

    assert_eq!(result.pages.len(), 1);
    let page = &result.pages[0];
    assert_eq!(page.width_mm, 210.0);
    assert_eq!(page.height_mm, 297.0);
}

#[test]
fn test_compute_layout_elements_within_page() {
    let template = simple_template();
    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let result = compute_layout(&template, &data, &fonts).unwrap();
    let page = &result.pages[0];

    // Should have at least root + title = 2 elements
    assert!(
        page.elements.len() >= 2,
        "Expected at least 2 elements, got {}",
        page.elements.len()
    );

    for el in &page.elements {
        // All positions should be non-negative
        assert!(
            el.x_mm >= 0.0,
            "Element {} has negative x: {}",
            el.id,
            el.x_mm
        );
        assert!(
            el.y_mm >= 0.0,
            "Element {} has negative y: {}",
            el.id,
            el.y_mm
        );
        // All dimensions should be non-negative
        assert!(
            el.width_mm >= 0.0,
            "Element {} has negative width: {}",
            el.id,
            el.width_mm
        );
        assert!(
            el.height_mm >= 0.0,
            "Element {} has negative height: {}",
            el.id,
            el.height_mm
        );
        // Elements should be within page bounds (with small tolerance for rounding)
        assert!(
            el.x_mm + el.width_mm <= page.width_mm + 1.0,
            "Element {} exceeds page width: x={}+w={} > {}",
            el.id,
            el.x_mm,
            el.width_mm,
            page.width_mm
        );
        assert!(
            el.y_mm + el.height_mm <= page.height_mm + 1.0,
            "Element {} exceeds page height: y={}+h={} > {}",
            el.id,
            el.y_mm,
            el.height_mm,
            page.height_mm
        );
    }
}

#[test]
fn test_compute_layout_text_content_resolved() {
    let template = simple_template();
    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let result = compute_layout(&template, &data, &fonts).unwrap();
    let page = &result.pages[0];

    let title = page.elements.iter().find(|e| e.id == "title").unwrap();
    match &title.content {
        Some(dreport_layout::ResolvedContent::Text { value }) => {
            assert_eq!(value, "Hello World");
        }
        other => panic!("Expected Text content, got {:?}", other),
    }
}

#[test]
fn test_compute_layout_with_data_binding() {
    let template = Template {
        id: "t1".to_string(),
        name: "Binding Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        header: None,
        footer: None,
        format_config: None,
        root: ContainerElement {
            id: "root".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 0.0,
            padding: Padding {
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            },
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle::default(),
            break_inside: "auto".to_string(),
            children: vec![TemplateElement::Text(TextElement {
                id: "bound_text".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Fr { value: 1.0 },
                    height: SizeValue::Auto,
                    ..Default::default()
                },
                style: TextStyle {
                    font_size: Some(12.0),
                    ..Default::default()
                },
                content: None,
                binding: ScalarBinding {
                    path: "company.name".to_string(),
                },
            })],
        },
    };

    let data = serde_json::json!({
        "company": { "name": "Acme Corp" }
    });
    let fonts = load_test_fonts();

    let result = compute_layout(&template, &data, &fonts).unwrap();
    let page = &result.pages[0];

    let bound = page
        .elements
        .iter()
        .find(|e| e.id == "bound_text")
        .unwrap();
    match &bound.content {
        Some(dreport_layout::ResolvedContent::Text { value }) => {
            assert_eq!(value, "Acme Corp");
        }
        other => panic!("Expected Text content, got {:?}", other),
    }
}

#[test]
fn test_compute_layout_multiple_children_ordering() {
    let template = Template {
        id: "t1".to_string(),
        name: "Order Test".to_string(),
        page: PageSettings {
            width: 210.0,
            height: 297.0,
        },
        fonts: vec!["Noto Sans".to_string()],
        header: None,
        footer: None,
        format_config: None,
        root: ContainerElement {
            id: "root".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 5.0,
            padding: Padding {
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            },
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle::default(),
            break_inside: "auto".to_string(),
            children: vec![
                TemplateElement::StaticText(StaticTextElement {
                    id: "first".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(12.0),
                        ..Default::default()
                    },
                    content: "First".to_string(),
                }),
                TemplateElement::StaticText(StaticTextElement {
                    id: "second".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                    style: TextStyle {
                        font_size: Some(12.0),
                        ..Default::default()
                    },
                    content: "Second".to_string(),
                }),
            ],
        },
    };

    let data = serde_json::json!({});
    let fonts = load_test_fonts();

    let result = compute_layout(&template, &data, &fonts).unwrap();
    let page = &result.pages[0];

    let first = page.elements.iter().find(|e| e.id == "first").unwrap();
    let second = page.elements.iter().find(|e| e.id == "second").unwrap();

    // In column direction, second should be below first
    assert!(
        second.y_mm > first.y_mm,
        "Second element (y={}) should be below first (y={})",
        second.y_mm,
        first.y_mm
    );
}
