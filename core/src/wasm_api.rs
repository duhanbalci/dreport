use wasm_bindgen::prelude::*;

use crate::models::Template;
use crate::template_to_typst::{self, RenderMode};

/// Template JSON + Data JSON → Typst markup (editör modu, layout query dahil)
#[wasm_bindgen(js_name = "templateToTypstEditor")]
pub fn template_to_typst_editor(template_json: &str, data_json: &str) -> Result<String, JsValue> {
    let template: Template = serde_json::from_str(template_json)
        .map_err(|e| JsValue::from_str(&format!("Template parse hatasi: {}", e)))?;
    let data: serde_json::Value = serde_json::from_str(data_json)
        .map_err(|e| JsValue::from_str(&format!("Data parse hatasi: {}", e)))?;

    Ok(template_to_typst::template_to_typst(&template, &data, RenderMode::Editor))
}

/// Template JSON + Data JSON → Typst markup (PDF modu, layout query yok)
#[wasm_bindgen(js_name = "templateToTypstPdf")]
pub fn template_to_typst_pdf(template_json: &str, data_json: &str) -> Result<String, JsValue> {
    let template: Template = serde_json::from_str(template_json)
        .map_err(|e| JsValue::from_str(&format!("Template parse hatasi: {}", e)))?;
    let data: serde_json::Value = serde_json::from_str(data_json)
        .map_err(|e| JsValue::from_str(&format!("Data parse hatasi: {}", e)))?;

    Ok(template_to_typst::template_to_typst(&template, &data, RenderMode::Pdf))
}
