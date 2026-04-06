use std::sync::Mutex;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::FontData;
use crate::text_measure::TextMeasureCache;

/// Font verileri — dinamik olarak eklenebilir (Mutex ile).
static FONTS: Mutex<Vec<FontData>> = Mutex::new(Vec::new());

/// Text ölçüm cache'i — layout call'ları arasında persist eder.
static TEXT_CACHE: Mutex<Option<TextMeasureCache>> = Mutex::new(None);

/// Barcode pixel cache — (format, value, width, height, include_text) → RGBA bytes (header dahil).
static BARCODE_CACHE: Mutex<Option<HashMap<BarcodeCacheKey, Vec<u8>>>> = Mutex::new(None);

#[derive(Clone, Eq, PartialEq, Hash)]
struct BarcodeCacheKey {
    format: String,
    value: String,
    width: u32,
    height: u32,
    include_text: bool,
}

/// Font verilerini yükle (ilk çağrıda mevcut fontları değiştirir).
/// `buffers`: Her font dosyasının raw bytes'ı
/// Font metadata (family, weight, italic) otomatik olarak TTF'den parse edilir.
#[wasm_bindgen(js_name = "loadFonts")]
pub fn load_fonts(buffers: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
    let mut fonts_lock = FONTS.lock().unwrap();

    let mut fonts: Vec<FontData> = Vec::with_capacity(buffers.len());
    for buf in buffers {
        let data = buf.to_vec();
        match FontData::from_bytes(data) {
            Some(fd) => fonts.push(fd),
            None => {
                // Skip unparseable fonts silently
            }
        }
    }

    *fonts_lock = fonts;

    // Text cache'i temizle (yeni fontlarla eski ölçümler geçersiz)
    *TEXT_CACHE.lock().unwrap() = None;

    Ok(())
}

/// Mevcut font setine yeni fontlar ekle (on-demand loading için).
/// Mevcut fontları korur, yenileri ekler. Aynı family+weight+italic varsa üzerine yazar.
#[wasm_bindgen(js_name = "addFonts")]
pub fn add_fonts(buffers: Vec<js_sys::Uint8Array>) -> Result<(), JsValue> {
    let mut fonts_lock = FONTS.lock().unwrap();

    for buf in buffers {
        let data = buf.to_vec();
        if let Some(fd) = FontData::from_bytes(data) {
            // Aynı variant varsa kaldır (üzerine yaz)
            fonts_lock.retain(|existing| {
                !(existing.family.eq_ignore_ascii_case(&fd.family)
                    && existing.weight == fd.weight
                    && existing.italic == fd.italic)
            });
            fonts_lock.push(fd);
        }
    }

    // Text cache'i temizle
    *TEXT_CACHE.lock().unwrap() = None;

    Ok(())
}

/// Yüklü font ailelerini JSON olarak döndür.
/// Frontend'in hangi fontların yüklü olduğunu bilmesi için.
#[wasm_bindgen(js_name = "getLoadedFonts")]
pub fn get_loaded_fonts() -> String {
    let fonts = FONTS.lock().unwrap();
    let mut families: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for fd in fonts.iter() {
        let entry = families.entry(fd.family.clone()).or_default();
        entry.push(serde_json::json!({
            "weight": fd.weight,
            "italic": fd.italic,
        }));
    }

    let result: Vec<serde_json::Value> = families
        .into_iter()
        .map(|(family, variants)| serde_json::json!({
            "family": family,
            "variants": variants,
        }))
        .collect();

    serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

/// Layout hesapla.
/// `template_json`: Template JSON string
/// `data_json`: Data JSON string
/// Dönen değer: LayoutResult JSON string
#[wasm_bindgen(js_name = "computeLayout")]
pub fn compute_layout_wasm(template_json: &str, data_json: &str) -> Result<String, JsValue> {
    let template: dreport_core::models::Template =
        serde_json::from_str(template_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let data: serde_json::Value =
        serde_json::from_str(data_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let fonts = FONTS.lock().unwrap();
    if fonts.is_empty() {
        return Err(JsValue::from_str("Fonts not loaded. Call loadFonts() first."));
    }

    // Text cache'i al (veya ilk kullanımda oluştur)
    let mut cache_guard = TEXT_CACHE.lock().unwrap();
    let text_cache = cache_guard.take().unwrap_or_default();

    let (result, new_cache) = crate::compute_layout_cached(&template, &data, &fonts, text_cache)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Güncel cache'i geri koy
    *cache_guard = Some(new_cache);

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

    let mut barcode_guard = BARCODE_CACHE.lock().unwrap();
    let cache = barcode_guard.get_or_insert_with(HashMap::new);

    // Cache hit?
    if let Some(cached_data) = cache.get(&cache_key) {
        let arr = js_sys::Uint8ClampedArray::new_with_length(cached_data.len() as u32);
        arr.copy_from(cached_data);
        return Ok(arr);
    }

    let fonts = FONTS.lock().unwrap();
    let fonts_slice: Option<&[FontData]> = if fonts.is_empty() { None } else { Some(&fonts) };
    let result = crate::barcode_gen::generate_barcode_pixels(format, value, width, height, include_text, fonts_slice)
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
    cache.insert(cache_key, data);

    Ok(arr)
}
