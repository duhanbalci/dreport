use dreport_core::models::*;
use serde_json::Value;
use std::collections::HashMap;

/// Şu anki tarihi verilen format string'ine göre formatla.
/// Desteklenen tokenlar: YYYY, MM, DD, HH, mm, ss
/// WASM'da js_sys::Date, native'de SystemTime kullanır.
fn format_current_date(fmt: &str) -> String {
    let (year, month, day, hour, minute, second) = current_datetime_parts();
    fmt.replace("YYYY", &format!("{:04}", year))
        .replace("MM", &format!("{:02}", month))
        .replace("DD", &format!("{:02}", day))
        .replace("HH", &format!("{:02}", hour))
        .replace("mm", &format!("{:02}", minute))
        .replace("ss", &format!("{:02}", second))
}

#[cfg(target_arch = "wasm32")]
fn current_datetime_parts() -> (i32, u32, u32, u32, u32, u32) {
    let d = js_sys::Date::new_0();
    (
        d.get_full_year() as i32,
        d.get_month() as u32 + 1, // JS months are 0-based
        d.get_date() as u32,
        d.get_hours() as u32,
        d.get_minutes() as u32,
        d.get_seconds() as u32,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn current_datetime_parts() -> (i32, u32, u32, u32, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple UTC date calculation (no timezone dependency)
    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;
    let hour = (time_of_day / 3600) as u32;
    let minute = ((time_of_day % 3600) / 60) as u32;
    let second = (time_of_day % 60) as u32;

    // Days since 1970-01-01 → year/month/day (civil calendar)
    // Algorithm from Howard Hinnant's chrono-compatible date library
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m, d, hour, minute, second)
}

/// Çözümlenmiş rich text span'ı
#[derive(Debug, Clone)]
pub struct ResolvedRichSpan {
    pub text: String,
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub color: Option<String>,
}

/// Her element ID'si için çözümlenmiş text içeriğini tutar.
/// Table ve barcode gibi özel tipler de burada çözülür.
#[derive(Debug, Clone)]
pub struct ResolvedData {
    /// element_id → çözümlenmiş text içeriği
    pub texts: HashMap<String, String>,
    /// element_id → çözümlenmiş tablo verileri (headers, rows)
    pub tables: HashMap<String, ResolvedTable>,
    /// element_id → çözümlenmiş barcode değeri
    pub barcodes: HashMap<String, String>,
    /// element_id → çözümlenmiş image src
    pub images: HashMap<String, String>,
    /// page_number element_id → format string (sayfa bölme sonrası çözülecek)
    pub page_number_formats: HashMap<String, String>,
    /// element_id → çözümlenmiş rich text span listesi
    pub rich_texts: HashMap<String, Vec<ResolvedRichSpan>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedTable {
    pub rows: Vec<Vec<String>>,
}

/// JSON path ile veri çek: "firma.unvan" → data["firma"]["unvan"]
fn resolve_path<'a>(data: &'a Value, path: &str) -> &'a Value {
    let mut current = data;
    for key in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(key).unwrap_or(&Value::Null),
            _ => &Value::Null,
        };
    }
    current
}

/// JSON Value → display string
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}

/// Template'deki tüm binding'leri çözümle.
pub fn resolve_template(template: &Template, data: &Value) -> ResolvedData {
    let mut resolved = ResolvedData {
        texts: HashMap::new(),
        tables: HashMap::new(),
        barcodes: HashMap::new(),
        images: HashMap::new(),
        page_number_formats: HashMap::new(),
        rich_texts: HashMap::new(),
    };
    if let Some(ref header) = template.header {
        resolve_element(&TemplateElement::Container(header.clone()), data, &mut resolved);
    }
    resolve_element(&TemplateElement::Container(template.root.clone()), data, &mut resolved);
    if let Some(ref footer) = template.footer {
        resolve_element(&TemplateElement::Container(footer.clone()), data, &mut resolved);
    }
    resolved
}

fn resolve_element(el: &TemplateElement, data: &Value, resolved: &mut ResolvedData) {
    match el {
        TemplateElement::StaticText(e) => {
            resolved.texts.insert(e.id.clone(), e.content.clone());
        }
        TemplateElement::Text(e) => {
            let bound_value = value_to_string(resolve_path(data, &e.binding.path));
            let text = match &e.content {
                Some(prefix) if !prefix.is_empty() => format!("{}{}", prefix, bound_value),
                _ => bound_value,
            };
            resolved.texts.insert(e.id.clone(), text);
        }
        TemplateElement::PageNumber(e) => {
            // Format string'i sakla — sayfa bölme sonrası gerçek değerlerle çözülecek
            let fmt = e.format.as_deref().unwrap_or("{current} / {total}").to_string();
            resolved.page_number_formats.insert(e.id.clone(), fmt.clone());
            // Placeholder koy (tek sayfalık fallback)
            resolved.texts.insert(e.id.clone(), fmt.replace("{current}", "1").replace("{total}", "1"));
        }
        TemplateElement::Barcode(e) => {
            let value = if let Some(binding) = &e.binding {
                value_to_string(resolve_path(data, &binding.path))
            } else {
                e.value.clone().unwrap_or_default()
            };
            resolved.barcodes.insert(e.id.clone(), value);
        }
        TemplateElement::Image(e) => {
            let src = if let Some(binding) = &e.binding {
                value_to_string(resolve_path(data, &binding.path))
            } else {
                e.src.clone().unwrap_or_default()
            };
            resolved.images.insert(e.id.clone(), src);
        }
        TemplateElement::RepeatingTable(e) => {
            let array = resolve_path(data, &e.data_source.path);
            let rows = match array {
                Value::Array(items) => {
                    items
                        .iter()
                        .map(|item| {
                            e.columns
                                .iter()
                                .map(|col| {
                                    let v = resolve_path(item, &col.field);
                                    value_to_string(v)
                                })
                                .collect()
                        })
                        .collect()
                }
                _ => vec![],
            };
            resolved.tables.insert(e.id.clone(), ResolvedTable { rows });
        }
        TemplateElement::Container(e) => {
            for child in &e.children {
                resolve_element(child, data, resolved);
            }
        }
        TemplateElement::CurrentDate(e) => {
            let fmt = e.format.as_deref().unwrap_or("DD.MM.YYYY");
            let text = format_current_date(fmt);
            resolved.texts.insert(e.id.clone(), text);
        }
        TemplateElement::Checkbox(e) => {
            let checked = if let Some(binding) = &e.binding {
                let val = resolve_path(data, &binding.path);
                match val {
                    Value::Bool(b) => *b,
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::String(s) => s == "true" || s == "1",
                    _ => false,
                }
            } else {
                e.checked.unwrap_or(false)
            };
            // Store as "true"/"false" string in texts map
            resolved.texts.insert(e.id.clone(), checked.to_string());
        }
        TemplateElement::CalculatedText(e) => {
            let result = crate::expr_eval::evaluate_expression(&e.expression, data);
            let formatted = crate::expr_eval::apply_format(&result, e.format.as_deref());
            resolved.texts.insert(e.id.clone(), formatted);
        }
        TemplateElement::RichText(e) => {
            let spans: Vec<ResolvedRichSpan> = e
                .content
                .iter()
                .map(|span| {
                    let text = if let Some(ref binding) = span.binding {
                        let bound = value_to_string(resolve_path(data, &binding.path));
                        match &span.text {
                            Some(prefix) if !prefix.is_empty() => format!("{}{}", prefix, bound),
                            _ => bound,
                        }
                    } else {
                        span.text.clone().unwrap_or_default()
                    };
                    ResolvedRichSpan {
                        text,
                        font_size: span.style.font_size.or(e.style.font_size),
                        font_weight: span.style.font_weight.clone().or(e.style.font_weight.clone()),
                        font_family: span.style.font_family.clone().or(e.style.font_family.clone()),
                        color: span.style.color.clone().or(e.style.color.clone()),
                    }
                })
                .collect();
            resolved.rich_texts.insert(e.id.clone(), spans);
        }
        TemplateElement::Line(_) => {}
        TemplateElement::Shape(_) => {}
        TemplateElement::PageBreak(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_simple() {
        let data: Value = serde_json::json!({"name": "test"});
        assert_eq!(value_to_string(resolve_path(&data, "name")), "test");
    }

    #[test]
    fn test_resolve_path_nested() {
        let data: Value = serde_json::json!({
            "firma": {
                "unvan": "Acme A.Ş.",
                "vergiNo": "123"
            }
        });
        assert_eq!(
            value_to_string(resolve_path(&data, "firma.unvan")),
            "Acme A.Ş."
        );
        assert_eq!(
            value_to_string(resolve_path(&data, "firma.vergiNo")),
            "123"
        );
    }

    #[test]
    fn test_resolve_path_missing() {
        let data: Value = serde_json::json!({"name": "test"});
        let result = resolve_path(&data, "nonexistent.path");
        assert!(result.is_null());
        assert_eq!(value_to_string(result), "");
    }

    #[test]
    fn test_resolve_path_deep_missing() {
        let data: Value = serde_json::json!({"a": {"b": 42}});
        let result = resolve_path(&data, "a.b.c.d");
        assert!(result.is_null());
    }

    #[test]
    fn test_value_to_string_types() {
        assert_eq!(value_to_string(&serde_json::json!("hello")), "hello");
        assert_eq!(value_to_string(&serde_json::json!(42)), "42");
        assert_eq!(value_to_string(&serde_json::json!(3.14)), "3.14");
        assert_eq!(value_to_string(&serde_json::json!(true)), "true");
        assert_eq!(value_to_string(&serde_json::json!(null)), "");
    }

    #[test]
    fn test_resolve_array() {
        let data: Value = serde_json::json!({
            "kalemler": [
                { "adi": "Widget", "tutar": 100 },
                { "adi": "Gadget", "tutar": 200 }
            ]
        });
        let arr = resolve_path(&data, "kalemler");
        assert!(arr.is_array());
        assert_eq!(arr.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_resolve_template_text_binding() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::Text(TextElement {
                        id: "el_name".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        style: TextStyle::default(),
                        content: None,
                        binding: ScalarBinding { path: "firma.unvan".to_string() },
                    }),
                ],
            },
        };

        let data = serde_json::json!({
            "firma": { "unvan": "Acme Teknoloji A.Ş." }
        });

        let resolved = resolve_template(&template, &data);
        assert_eq!(
            resolved.texts.get("el_name").unwrap(),
            "Acme Teknoloji A.Ş."
        );
    }

    #[test]
    fn test_resolve_template_text_with_prefix() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::Text(TextElement {
                        id: "el_no".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        style: TextStyle::default(),
                        content: Some("Fatura No: ".to_string()),
                        binding: ScalarBinding { path: "fatura.no".to_string() },
                    }),
                ],
            },
        };

        let data = serde_json::json!({
            "fatura": { "no": "FTR-001" }
        });

        let resolved = resolve_template(&template, &data);
        assert_eq!(
            resolved.texts.get("el_no").unwrap(),
            "Fatura No: FTR-001"
        );
    }

    #[test]
    fn test_resolve_template_static_text() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::StaticText(StaticTextElement {
                        id: "title".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        style: TextStyle::default(),
                        content: "FATURA".to_string(),
                    }),
                ],
            },
        };

        let resolved = resolve_template(&template, &serde_json::json!({}));
        assert_eq!(resolved.texts.get("title").unwrap(), "FATURA");
    }

    #[test]
    fn test_resolve_template_table_binding() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::RepeatingTable(RepeatingTableElement {
                        id: "tbl".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        data_source: ArrayBinding { path: "kalemler".to_string() },
                        columns: vec![
                            TableColumn {
                                id: "col_adi".to_string(),
                                field: "adi".to_string(),
                                title: "Urun Adi".to_string(),
                                width: SizeValue::Fr { value: 1.0 },
                                align: "left".to_string(),
                                format: None,
                            },
                            TableColumn {
                                id: "col_tutar".to_string(),
                                field: "tutar".to_string(),
                                title: "Tutar".to_string(),
                                width: SizeValue::Fixed { value: 30.0 },
                                align: "right".to_string(),
                                format: None,
                            },
                        ],
                        style: TableStyle::default(),
                        repeat_header: Some(true),
                    }),
                ],
            },
        };

        let data = serde_json::json!({
            "kalemler": [
                { "adi": "Widget", "tutar": 100 },
                { "adi": "Gadget", "tutar": 200 }
            ]
        });

        let resolved = resolve_template(&template, &data);
        let table = resolved.tables.get("tbl").unwrap();
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Widget", "100"]);
        assert_eq!(table.rows[1], vec!["Gadget", "200"]);
    }

    #[test]
    fn test_resolve_template_table_empty_array() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::RepeatingTable(RepeatingTableElement {
                        id: "tbl".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        data_source: ArrayBinding { path: "items".to_string() },
                        columns: vec![
                            TableColumn {
                                id: "c1".to_string(),
                                field: "name".to_string(),
                                title: "Name".to_string(),
                                width: SizeValue::Fr { value: 1.0 },
                                align: "left".to_string(),
                                format: None,
                            },
                        ],
                        style: TableStyle::default(),
                        repeat_header: Some(true),
                    }),
                ],
            },
        };

        let data = serde_json::json!({ "items": [] });
        let resolved = resolve_template(&template, &data);
        let table = resolved.tables.get("tbl").unwrap();
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn test_resolve_template_missing_binding_path() {
        let template = Template {
            id: "t1".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint::default(),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding::default(),
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::Text(TextElement {
                        id: "el_missing".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint::default(),
                        style: TextStyle::default(),
                        content: None,
                        binding: ScalarBinding { path: "does.not.exist".to_string() },
                    }),
                ],
            },
        };

        let data = serde_json::json!({});
        let resolved = resolve_template(&template, &data);
        // Missing binding path should resolve to empty string
        assert_eq!(resolved.texts.get("el_missing").unwrap(), "");
    }
}
