//! Application bootstrap. Builds a fully-configured `DreportService` for the
//! HTTP layer (and tests) to share.

use anyhow::Result;
use dreport_service::DreportService;

/// Construct the service used by the running server. Loads embedded fonts
/// (compile-time defaults) and any extra fonts in `DREPORT_FONTS_DIR`.
pub fn build_service() -> Result<DreportService> {
    let svc = DreportService::new();
    if let Ok(dir) = std::env::var("DREPORT_FONTS_DIR") {
        match svc.register_fonts_directory(&dir) {
            Ok(n) => println!("DREPORT_FONTS_DIR'den {} font yüklendi: {}", n, dir),
            Err(e) => eprintln!("DREPORT_FONTS_DIR yüklenemedi ({}): {}", dir, e),
        }
    }
    Ok(svc)
}
