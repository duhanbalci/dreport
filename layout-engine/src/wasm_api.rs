use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::FontData;
use crate::text_measure::TextMeasureCache;

/// Font verileri worker'da cache'lenir.
static FONTS: OnceLock<Vec<FontData>> = OnceLock::new();

/// Text ölçüm cache'i — layout call'ları arasında persist eder.
/// Aynı text + font + size + weight + available_width → aynı sonuç.
static TEXT_CACHE: OnceLock<Mutex<TextMeasureCache>> = OnceLock::new();

/// Barcode pixel cache — (format, value, width, height, include_text) → RGBA bytes (header dahil).
static BARCODE_CACHE: OnceLock<Mutex<HashMap<BarcodeCacheKey, Vec<u8>>>> = OnceLock::new();

#[derive(Clone, Eq, PartialEq, Hash)]
struct BarcodeCacheKey {
    format: String,
    value: String,
    width: u32,
    height: u32,
    include_text: bool,
}

/// Font verilerini yükle (worker init sırasında bir kere çağrılır).
/// `families`: JSON array of font family names — ["Noto Sans", "Noto Sans", ...]
/// `buffers`: Her font dosyasının raw bytes'ı (sırayla)
#[wasm_bindgen(js_name = "loadFonts")]
pub fn load_fonts(families: &str, buffers: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
    let families: Vec<String> =
        serde_json::from_str(families).map_err(|e| JsValue::from_str(&e.to_string()))?;

    if families.len() != buffers.len() {
        return Err(JsValue::from_str("families and buffers length mismatch"));
    }

    let fonts: Vec<FontData> = families
        .into_iter()
        .zip(buffers.into_iter())
        .map(|(family, buf)| FontData {
            family,
            data: buf.to_vec(),
        })
        .collect();

    FONTS
        .set(fonts)
        .map_err(|_| JsValue::from_str("Fonts already loaded"))?;

    Ok(())
}

/// Layout hesapla.
/// `template_json`: Template JSON string
/// `data_json`: Data JSON string
/// Dönen değer: LayoutResult JSON string
///
/// Text ölçüm sonuçları cross-call cache'lenir — değişmeyen text elemanları
/// cosmic-text'e gitmeden cache'ten döner.
#[wasm_bindgen(js_name = "computeLayout")]
pub fn compute_layout_wasm(template_json: &str, data_json: &str) -> Result<String, JsValue> {
    let template: dreport_core::models::Template =
        serde_json::from_str(template_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let data: serde_json::Value =
        serde_json::from_str(data_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let fonts = FONTS
        .get()
        .ok_or_else(|| JsValue::from_str("Fonts not loaded. Call loadFonts() first."))?;

    // Text cache'i al (veya ilk kullanımda oluştur)
    let cache_mutex = TEXT_CACHE.get_or_init(|| Mutex::new(TextMeasureCache::default()));
    let text_cache = cache_mutex.lock().unwrap().take();

    let (result, new_cache) = crate::compute_layout_cached(&template, &data, fonts, text_cache);

    // Güncel cache'i geri koy
    *cache_mutex.lock().unwrap() = new_cache;

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Barcode üret → ham RGBA pixel verisi (header: 8 byte width+height LE, sonra RGBA).
/// Sonuç cache'lenir — aynı parametrelerle tekrar çağrılırsa cache'ten döner.
#[wasm_bindgen(js_name = "generateBarcode")]
pub fn generate_barcode_wasm(format: &str, value: &str, width: u32, height: u32, include_text: bool) -> Result<js_sys::Uint8ClampedArray, JsValue> {
    let cache_key = BarcodeCacheKey {
        format: format.to_string(),
        value: value.to_string(),
        width,
        height,
        include_text,
    };

    let cache_mutex = BARCODE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Cache hit?
    {
        let cache = cache_mutex.lock().unwrap();
        if let Some(cached_data) = cache.get(&cache_key) {
            let arr = js_sys::Uint8ClampedArray::new_with_length(cached_data.len() as u32);
            arr.copy_from(cached_data);
            return Ok(arr);
        }
    }

    // Cache miss — üret
    let fonts = FONTS.get().map(|f| f.as_slice());
    let result = crate::barcode_gen::generate_barcode_pixels(format, value, width, height, include_text, fonts)
        .map_err(|e| JsValue::from_str(&e))?;

    // Grayscale → RGBA (canvas ImageData formatı)
    let mut rgba = Vec::with_capacity((result.width * result.height * 4) as usize);
    for &gray in &result.pixels {
        rgba.push(gray); // R
        rgba.push(gray); // G
        rgba.push(gray); // B
        rgba.push(255);  // A
    }

    // Header (8 byte: width LE + height LE) + RGBA pixel verisi
    let mut data = Vec::with_capacity(8 + rgba.len());
    data.extend_from_slice(&result.width.to_le_bytes());
    data.extend_from_slice(&result.height.to_le_bytes());
    data.extend_from_slice(&rgba);

    let arr = js_sys::Uint8ClampedArray::new_with_length(data.len() as u32);
    arr.copy_from(&data);

    // Cache'e kaydet
    cache_mutex.lock().unwrap().insert(cache_key, data);

    Ok(arr)
}
