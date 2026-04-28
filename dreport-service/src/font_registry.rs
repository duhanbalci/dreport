use dreport_layout::FontData;
use dreport_layout::font_meta::{self, FontFamilyInfo, FontVariantKey};
use dreport_layout::font_provider::FontProvider;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ServiceError, ServiceResult};

/// Default font family that is always included in the layout font set when
/// available. Matches the engine's fallback behaviour.
pub(crate) const DEFAULT_FAMILY: &str = "noto sans";

/// Internal font registry. Manages parsed TTF/OTF faces indexed by family + variant.
/// Not exported directly — accessed through `DreportService`.
#[derive(Default)]
pub(crate) struct FontRegistry {
    /// family_lower -> variant_key -> FontData
    families: HashMap<String, HashMap<FontVariantKey, FontData>>,
    /// Original-case family names for display (`list_families`).
    family_names: HashMap<String, String>,
}

impl FontRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Register a font from raw bytes. Returns parsed family info on success.
    pub(crate) fn register_bytes(&mut self, data: Vec<u8>) -> ServiceResult<RegisteredFont> {
        let meta = font_meta::parse_font_meta(&data).ok_or(ServiceError::FontParseFailed)?;
        let family_lower = meta.family.to_lowercase();
        let variant_key = meta.variant_key();

        self.family_names
            .entry(family_lower.clone())
            .or_insert_with(|| meta.family.clone());

        let font_data = FontData::new(meta.family.clone(), meta.weight, meta.italic, data);
        self.families
            .entry(family_lower)
            .or_default()
            .insert(variant_key.clone(), font_data);

        Ok(RegisteredFont {
            family: meta.family,
            weight: variant_key.weight,
            italic: variant_key.italic,
        })
    }

    /// Register all `.ttf`/`.otf` files in the given directory.
    /// Returns the count of successfully registered files; per-file parse
    /// failures are silently skipped to mirror the previous backend behaviour.
    pub(crate) fn register_directory(&mut self, dir: &Path) -> ServiceResult<usize> {
        if !dir.exists() {
            return Err(ServiceError::FontDirNotFound(dir.display().to_string()));
        }
        if !dir.is_dir() {
            return Err(ServiceError::FontDirNotFound(dir.display().to_string()));
        }

        let entries = std::fs::read_dir(dir).map_err(|e| ServiceError::FontDirRead(e.to_string()))?;

        let mut count = 0_usize;
        for entry in entries.flatten() {
            let path = entry.path();
            let is_font = path
                .extension()
                .is_some_and(|e| e == "ttf" || e == "otf" || e == "TTF" || e == "OTF");
            if !is_font {
                continue;
            }
            if let Ok(data) = std::fs::read(&path)
                && self.register_bytes(data).is_ok()
            {
                count += 1;
            }
        }
        Ok(count)
    }

    pub(crate) fn get_font_bytes(
        &self,
        family: &str,
        weight: u16,
        italic: bool,
    ) -> Option<&[u8]> {
        let family_lower = family.to_lowercase();
        let key = FontVariantKey { weight, italic };
        self.families
            .get(&family_lower)
            .and_then(|variants| variants.get(&key))
            .map(|fd| fd.data.as_slice())
    }

    /// Resolve the FontData set for a template. Always includes the default
    /// family (Noto Sans) plus any explicitly requested families.
    pub(crate) fn fonts_for_families(&self, families: &[String]) -> Vec<FontData> {
        let mut result = Vec::new();
        let mut loaded: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut to_load: Vec<String> = vec![DEFAULT_FAMILY.to_string()];
        for f in families {
            let fl = f.to_lowercase();
            if !to_load.contains(&fl) {
                to_load.push(fl);
            }
        }

        for family_lower in &to_load {
            if !loaded.insert(family_lower.clone()) {
                continue;
            }
            if let Some(variants) = self.families.get(family_lower) {
                for fd in variants.values() {
                    result.push(fd.clone());
                }
            }
        }

        result
    }

    pub(crate) fn family_count(&self) -> usize {
        self.families.len()
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

/// Result of registering a single font, returned to callers that need to
/// confirm what variant was actually parsed.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisteredFont {
    pub family: String,
    pub weight: u16,
    pub italic: bool,
}
