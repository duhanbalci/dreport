use dreport_core::models::*;
use serde_json::Value;
use std::collections::HashMap;

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
    };
    resolve_element(&TemplateElement::Container(template.root.clone()), data, &mut resolved);
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
            // Sayfa numarası layout sonrasında çözülecek, placeholder koy
            let fmt = e.format.as_deref().unwrap_or("{current} / {total}");
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
        TemplateElement::Line(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
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
        assert_eq!(
            value_to_string(resolve_path(&data, "nonexistent.path")),
            ""
        );
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
}
