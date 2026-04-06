use dreport_layout::FontData;
use dreport_layout::font_meta::{self, FontFamilyInfo, FontVariantKey};
use dreport_layout::font_provider::FontProvider;
use std::collections::HashMap;

/// Font registry — manages all available fonts from embedded defaults + external directory.
pub struct FontRegistry {
    /// family_lower -> variant_key -> FontData
    families: HashMap<String, HashMap<FontVariantKey, FontData>>,
    /// Original-case family names
    family_names: HashMap<String, String>,
}

impl FontRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            families: HashMap::new(),
            family_names: HashMap::new(),
        };

        // Load embedded default fonts
        registry.load_embedded_defaults();

        // Load fonts from DREPORT_FONTS_DIR if set
        if let Ok(dir) = std::env::var("DREPORT_FONTS_DIR") {
            registry.load_from_directory(&dir);
        }

        registry
    }

    fn load_embedded_defaults(&mut self) {
        let embedded: &[(&str, &[u8])] = &[
            (
                "NotoSans-Regular",
                include_bytes!("../fonts/NotoSans-Regular.ttf"),
            ),
            (
                "NotoSans-Bold",
                include_bytes!("../fonts/NotoSans-Bold.ttf"),
            ),
            (
                "NotoSans-Italic",
                include_bytes!("../fonts/NotoSans-Italic.ttf"),
            ),
            (
                "NotoSans-BoldItalic",
                include_bytes!("../fonts/NotoSans-BoldItalic.ttf"),
            ),
            (
                "NotoSansMono-Regular",
                include_bytes!("../fonts/NotoSansMono-Regular.ttf"),
            ),
        ];

        for (_name, data) in embedded {
            self.register_font(data.to_vec());
        }
    }

    fn load_from_directory(&mut self, dir: &str) {
        let path = std::path::Path::new(dir);
        if !path.is_dir() {
            eprintln!("DREPORT_FONTS_DIR dizini bulunamadı: {}", dir);
            return;
        }

        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("DREPORT_FONTS_DIR okunamadı: {}", e);
                return;
            }
        };

        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "ttf" || e == "otf")
                && let Ok(data) = std::fs::read(&p)
            {
                if self.register_font(data) {
                    println!("  Font yüklendi: {}", p.display());
                } else {
                    eprintln!("  Font parse edilemedi: {}", p.display());
                }
            }
        }
    }

    /// Register a font from raw bytes. Returns true if successful.
    fn register_font(&mut self, data: Vec<u8>) -> bool {
        let Some(meta) = font_meta::parse_font_meta(&data) else {
            return false;
        };

        let family_lower = meta.family.to_lowercase();
        let variant_key = meta.variant_key();

        self.family_names
            .entry(family_lower.clone())
            .or_insert_with(|| meta.family.clone());

        let font_data = FontData::new(meta.family, meta.weight, meta.italic, data);

        self.families
            .entry(family_lower)
            .or_default()
            .insert(variant_key, font_data);

        true
    }

    /// Get a specific font's raw bytes
    pub fn get_font_bytes(&self, family: &str, weight: u16, italic: bool) -> Option<&[u8]> {
        let family_lower = family.to_lowercase();
        let key = FontVariantKey { weight, italic };
        self.families
            .get(&family_lower)
            .and_then(|variants| variants.get(&key))
            .map(|fd| fd.data.as_slice())
    }

    /// Get all FontData for given family names (for passing to layout engine)
    pub fn fonts_for_families(&self, families: &[String]) -> Vec<FontData> {
        let mut result = Vec::new();
        let mut loaded = std::collections::HashSet::new();

        // Always include default family
        let default_lower = "noto sans".to_string();
        let mut to_load: Vec<String> = vec![default_lower.clone()];
        for f in families {
            let fl = f.to_lowercase();
            if !to_load.contains(&fl) {
                to_load.push(fl);
            }
        }

        for family_lower in &to_load {
            if loaded.contains(family_lower) {
                continue;
            }
            if let Some(variants) = self.families.get(family_lower) {
                for fd in variants.values() {
                    result.push(fd.clone());
                }
                loaded.insert(family_lower.clone());
            }
        }

        result
    }
}

impl FontProvider for FontRegistry {
    fn list_families(&self) -> Vec<FontFamilyInfo> {
        self.families
            .iter()
            .map(|(family_lower, variants)| {
                let family = self
                    .family_names
                    .get(family_lower)
                    .cloned()
                    .unwrap_or_else(|| family_lower.clone());
                FontFamilyInfo {
                    family,
                    variants: variants.keys().cloned().collect(),
                }
            })
            .collect()
    }

    fn load_font(&self, family: &str, weight: u16, italic: bool) -> Option<FontData> {
        let family_lower = family.to_lowercase();
        let key = FontVariantKey { weight, italic };
        self.families
            .get(&family_lower)
            .and_then(|variants| variants.get(&key))
            .cloned()
    }
}
