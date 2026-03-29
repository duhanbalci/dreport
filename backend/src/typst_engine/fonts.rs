use std::path::PathBuf;
use typst_kit::fonts::{FontSearcher, Fonts};

/// Proje fontlarını yükler (backend/fonts/ dizininden).
/// Uygulama başlangıcında bir kez çağrılır ve paylaşılır.
pub fn load_fonts() -> Fonts {
    let font_dir = font_dir();
    FontSearcher::new()
        .include_system_fonts(false)
        .search_with(&[font_dir])
}

fn font_dir() -> PathBuf {
    // Cargo manifest dizinine göre fonts/ klasörü
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("fonts")
}
