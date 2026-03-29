use std::collections::HashMap;
use std::hash::Hash;

use crate::FontData;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};

/// Opak text ölçüm cache'i. `TextMeasurer` call'ları arasında taşınarak
/// aynı parametrelerle yapılan ölçümlerin yeniden hesaplanmasını önler.
#[derive(Default)]
pub struct TextMeasureCache {
    entries: HashMap<MeasureCacheKey, (f32, f32)>,
}

impl TextMeasureCache {
    /// Cache içeriğini al ve yerine boş cache bırak.
    pub fn take(&mut self) -> Self {
        Self {
            entries: std::mem::take(&mut self.entries),
        }
    }
}

/// Cache key — text ölçüm parametrelerinin hash'lenebilir temsili.
/// f32 değerler bit-exact karşılaştırma için u32'ye çevrilir.
#[derive(Clone, Eq, PartialEq, Hash)]
struct MeasureCacheKey {
    text: String,
    font_family: Option<String>,
    font_size_bits: u32,
    font_weight: Option<String>,
    available_width_bits: Option<u32>,
}

impl MeasureCacheKey {
    fn new(
        text: &str,
        font_family: Option<&str>,
        font_size_pt: f32,
        font_weight: Option<&str>,
        available_width_pt: Option<f32>,
    ) -> Self {
        Self {
            text: text.to_string(),
            font_family: font_family.map(|s| s.to_string()),
            font_size_bits: font_size_pt.to_bits(),
            font_weight: font_weight.map(|s| s.to_string()),
            available_width_bits: available_width_pt.map(|w| w.to_bits()),
        }
    }
}

/// Text ölçüm motoru. cosmic-text kullanarak verilen font, boyut ve
/// mevcut genişlik kısıtı ile text'in kaplayacağı alanı hesaplar.
/// Ölçüm sonuçları cache'lenir — aynı parametrelerle tekrar çağrılırsa
/// cosmic-text'e gitmeden cache'ten döner.
pub struct TextMeasurer {
    font_system: FontSystem,
    cache: HashMap<MeasureCacheKey, (f32, f32)>,
}

/// pt → px dönüşümü (cosmic-text px cinsinden çalışır, 1pt = 1.333px @96dpi)
const PT_TO_PX: f32 = 96.0 / 72.0;

impl TextMeasurer {
    pub fn new(fonts: &[FontData]) -> Self {
        let mut font_system = FontSystem::new_with_locale_and_db(
            "tr-TR".to_string(),
            cosmic_text::fontdb::Database::new(),
        );
        for font in fonts {
            font_system.db_mut().load_font_data(font.data.clone());
        }
        Self {
            font_system,
            cache: HashMap::new(),
        }
    }

    /// Mevcut cache'i koruyarak yeni bir TextMeasurer oluştur.
    /// Font seti değişmediyse eski cache geçerliliğini korur.
    pub fn new_with_cache(fonts: &[FontData], cache: TextMeasureCache) -> Self {
        let mut m = Self::new(fonts);
        m.cache = cache.entries;
        m
    }

    /// Cache'i dışarı taşı (persist etmek için).
    pub fn take_cache(self) -> TextMeasureCache {
        TextMeasureCache { entries: self.cache }
    }

    /// Text'i ölç. Dönen değerler pt cinsinden (width, height).
    /// `available_width_pt`: Mevcut genişlik kısıtı (pt). None ise sınırsız.
    /// Sonuç cache'lenir — aynı parametrelerle tekrar çağrılırsa cache'ten döner.
    pub fn measure(
        &mut self,
        text: &str,
        font_family: Option<&str>,
        font_size_pt: f32,
        font_weight: Option<&str>,
        available_width_pt: Option<f32>,
    ) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, font_size_pt * 1.2);
        }

        let key = MeasureCacheKey::new(text, font_family, font_size_pt, font_weight, available_width_pt);

        if let Some(&cached) = self.cache.get(&key) {
            return cached;
        }

        let result = self.measure_uncached(text, font_family, font_size_pt, font_weight, available_width_pt);
        self.cache.insert(key, result);
        result
    }

    /// Cache'siz ölçüm — cosmic-text ile gerçek hesaplama.
    fn measure_uncached(
        &mut self,
        text: &str,
        font_family: Option<&str>,
        font_size_pt: f32,
        font_weight: Option<&str>,
        available_width_pt: Option<f32>,
    ) -> (f32, f32) {
        let font_size_px = font_size_pt * PT_TO_PX;
        let line_height_px = font_size_px * 1.2;
        let metrics = Metrics::new(font_size_px, line_height_px);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let width_px = available_width_pt.map(|w| w * PT_TO_PX);
        buffer.set_size(&mut self.font_system, width_px, None);

        let weight = match font_weight {
            Some("bold") => Weight::BOLD,
            _ => Weight::NORMAL,
        };

        let family_name = font_family.unwrap_or("Noto Sans");
        let attrs = Attrs::new()
            .family(Family::Name(family_name))
            .weight(weight);

        buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut max_width: f32 = 0.0;
        let mut total_height: f32 = 0.0;

        for run in buffer.layout_runs() {
            let run_width = run.line_w;
            if run_width > max_width {
                max_width = run_width;
            }
            total_height = run.line_top + line_height_px;
        }

        if total_height == 0.0 {
            total_height = line_height_px;
        }

        let width_pt = max_width / PT_TO_PX;
        let height_pt = total_height / PT_TO_PX;

        // Text genişliğine küçük bir tolerans ekle (0.5pt ≈ 0.18mm).
        // cosmic-text ile browser font engine'i farklı subpixel sonuçlar üretir;
        // bu fark zoom değişimlerinde text wrap sınırında flickering'e yol açar.
        // 0.5pt baskıda görünmez ama wrapping dengesizliğini önler.
        let width_pt = width_pt + 0.5;

        (width_pt, height_pt)
    }
}

#[cfg(test)]
pub(crate) fn load_test_fonts() -> Vec<crate::FontData> {
    let font_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("backend/fonts");

    let mut fonts = Vec::new();
    for entry in std::fs::read_dir(&font_dir).expect("backend/fonts dizini bulunamadı") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ttf") {
            let data = std::fs::read(&path).unwrap();
            let family = if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("Mono")
            {
                "Noto Sans Mono".to_string()
            } else {
                "Noto Sans".to_string()
            };
            fonts.push(crate::FontData { family, data });
        }
    }
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_measurer() -> TextMeasurer {
        TextMeasurer::new(&load_test_fonts())
    }

    #[test]
    fn test_empty_text() {
        let mut m = make_measurer();
        let (w, h) = m.measure("", None, 12.0, None, None);
        assert_eq!(w, 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn test_basic_measurement() {
        let mut m = make_measurer();
        let (w, h) = m.measure("Hello", None, 12.0, None, None);
        assert!(w > 0.0, "Width should be positive, got {w}");
        assert!(h > 0.0, "Height should be positive, got {h}");
    }

    #[test]
    fn test_cache_returns_same_result() {
        let mut m = make_measurer();
        let (w1, h1) = m.measure("Cache test", None, 14.0, Some("bold"), Some(100.0));
        let (w2, h2) = m.measure("Cache test", None, 14.0, Some("bold"), Some(100.0));
        assert_eq!(w1, w2);
        assert_eq!(h1, h2);
        // Cache'te 1 entry olmalı (aynı key iki kere çağrıldı)
        assert_eq!(m.cache.len(), 1);
    }

    #[test]
    fn test_cache_persists_across_measurers() {
        let fonts = load_test_fonts();
        let mut m1 = TextMeasurer::new(&fonts);
        let (w1, h1) = m1.measure("Persist test", None, 12.0, None, None);
        let cache = m1.take_cache();

        let mut m2 = TextMeasurer::new_with_cache(&fonts, cache);
        assert_eq!(m2.cache.len(), 1);
        let (w2, h2) = m2.measure("Persist test", None, 12.0, None, None);
        assert_eq!(w1, w2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_wrapping_reduces_width() {
        let mut m = make_measurer();
        // Sınırsız genişlikte ölç
        let (w_unlimited, h_unlimited) =
            m.measure("This is a longer text that should wrap", None, 12.0, None, None);
        // Dar genişlikte ölç
        let (w_narrow, h_narrow) =
            m.measure("This is a longer text that should wrap", None, 12.0, None, Some(50.0));

        // Dar genişlikte yükseklik artmalı (wrapping oldu)
        assert!(
            h_narrow >= h_unlimited,
            "Wrapped height ({h_narrow}) should be >= unlimited height ({h_unlimited})"
        );
        // Dar genişlikte genişlik kısıtlanmış olmalı
        assert!(
            w_narrow <= w_unlimited + 1.0,
            "Wrapped width ({w_narrow}) should be <= unlimited width ({w_unlimited})"
        );
    }
}
