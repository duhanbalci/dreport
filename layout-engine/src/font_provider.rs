use crate::FontData;
use crate::font_meta::FontFamilyInfo;

/// Font resolution trait — host apps implement this to provide fonts.
/// Backend implements it with a file-based registry, WASM side with API fetching.
pub trait FontProvider: Send + Sync {
    /// List all available font families with their variants.
    fn list_families(&self) -> Vec<FontFamilyInfo>;

    /// Load a specific font variant. Returns None if not found.
    fn load_font(&self, family: &str, weight: u16, italic: bool) -> Option<FontData>;

    /// The default/fallback font family name.
    fn default_family(&self) -> &str {
        "Noto Sans"
    }

    /// Load all variants of the given families. Falls back to default family if a family is not found.
    /// Always includes the default family.
    fn load_families(&self, families: &[String]) -> Vec<FontData> {
        let mut result = Vec::new();
        let mut loaded_families = std::collections::HashSet::new();

        // Always include default family
        let mut all_families: Vec<String> = vec![self.default_family().to_string()];
        for f in families {
            if !all_families.iter().any(|af| af.eq_ignore_ascii_case(f)) {
                all_families.push(f.clone());
            }
        }

        for family in &all_families {
            let family_lower = family.to_lowercase();
            if loaded_families.contains(&family_lower) {
                continue;
            }

            let infos = self.list_families();
            if let Some(info) = infos
                .iter()
                .find(|i| i.family.to_lowercase() == family_lower)
            {
                for variant in &info.variants {
                    if let Some(fd) = self.load_font(&info.family, variant.weight, variant.italic) {
                        result.push(fd);
                    }
                }
                loaded_families.insert(family_lower);
            }
        }

        result
    }
}
