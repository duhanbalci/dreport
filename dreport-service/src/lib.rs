//! dreport-service
//!
//! High-level orchestration layer that sits on top of `dreport-layout`.
//! Responsible for:
//! - Font registry management (embedded defaults + external loading)
//! - Template + data → LayoutResult JSON
//! - Template + data → PDF bytes
//!
//! Consumed by:
//! - `dreport-backend` (Axum HTTP adapter)
//! - `dreport-ffi` (C ABI for NuGet etc.)
//! - Any other Rust host (CLI, gRPC, ...)

mod error;
mod font_registry;

pub use dreport_core::models::Template;
pub use dreport_layout::FontData;
pub use dreport_layout::LayoutResult;
pub use dreport_layout::font_meta::{FontFamilyInfo, FontVariantKey};
pub use dreport_layout::font_provider::FontProvider;
pub use error::{ServiceError, ServiceResult};
pub use font_registry::RegisteredFont;

use std::path::Path;
use std::sync::RwLock;

use font_registry::FontRegistry;

/// Embedded default fonts compiled into the binary when the
/// `embedded-fonts` feature is enabled (default).
#[cfg(feature = "embedded-fonts")]
const EMBEDDED_FONTS: &[(&str, &[u8])] = &[
    (
        "NotoSans-Regular",
        include_bytes!("../assets/fonts/NotoSans-Regular.ttf"),
    ),
    (
        "NotoSans-Bold",
        include_bytes!("../assets/fonts/NotoSans-Bold.ttf"),
    ),
    (
        "NotoSans-Italic",
        include_bytes!("../assets/fonts/NotoSans-Italic.ttf"),
    ),
    (
        "NotoSans-BoldItalic",
        include_bytes!("../assets/fonts/NotoSans-BoldItalic.ttf"),
    ),
    (
        "NotoSansMono-Regular",
        include_bytes!("../assets/fonts/NotoSansMono-Regular.ttf"),
    ),
];

/// Main service handle. Thread-safe; share across threads via `Arc`.
///
/// Holds the font registry and exposes layout + PDF rendering operations.
/// All mutating operations (font registration) take `&self` and use internal
/// synchronization, so multiple readers (renders) and writers (font loads)
/// can coexist safely.
pub struct DreportService {
    registry: RwLock<FontRegistry>,
}

impl DreportService {
    /// Create a new service. Embedded default fonts are loaded automatically
    /// when the `embedded-fonts` feature is on (default).
    pub fn new() -> Self {
        let mut reg = FontRegistry::new();
        #[cfg(feature = "embedded-fonts")]
        for (_name, bytes) in EMBEDDED_FONTS {
            // Embedded fonts must parse — failure is a build-time bug.
            let _ = reg.register_bytes(bytes.to_vec());
        }
        Self {
            registry: RwLock::new(reg),
        }
    }

    /// Create a service without the embedded defaults, regardless of feature
    /// flags. Useful for tests and minimal embedders.
    pub fn empty() -> Self {
        Self {
            registry: RwLock::new(FontRegistry::new()),
        }
    }

    // -----------------------------------------------------------------
    // Font registry operations
    // -----------------------------------------------------------------

    /// Register a single font from raw TTF/OTF bytes.
    pub fn register_font_bytes(&self, data: Vec<u8>) -> ServiceResult<RegisteredFont> {
        let mut reg = self.registry.write().expect("font registry poisoned");
        reg.register_bytes(data)
    }

    /// Register every `.ttf` / `.otf` file in `dir` (non-recursive).
    /// Returns the number of fonts successfully registered.
    pub fn register_fonts_directory<P: AsRef<Path>>(&self, dir: P) -> ServiceResult<usize> {
        let mut reg = self.registry.write().expect("font registry poisoned");
        reg.register_directory(dir.as_ref())
    }

    /// List all currently-registered font families with their available variants.
    pub fn list_font_families(&self) -> Vec<FontFamilyInfo> {
        let reg = self.registry.read().expect("font registry poisoned");
        reg.list_families()
    }

    /// Get the raw bytes for a specific font variant.
    pub fn get_font_bytes(&self, family: &str, weight: u16, italic: bool) -> Option<Vec<u8>> {
        let reg = self.registry.read().expect("font registry poisoned");
        reg.get_font_bytes(family, weight, italic).map(<[u8]>::to_vec)
    }

    /// Number of distinct font families currently registered.
    pub fn font_family_count(&self) -> usize {
        let reg = self.registry.read().expect("font registry poisoned");
        reg.family_count()
    }

    // -----------------------------------------------------------------
    // Render pipeline
    // -----------------------------------------------------------------

    /// Compute layout from JSON inputs. Returns the LayoutResult serialized as JSON.
    pub fn compute_layout_json(
        &self,
        template_json: &str,
        data_json: &str,
    ) -> ServiceResult<String> {
        let template: Template = serde_json::from_str(template_json)
            .map_err(|e| ServiceError::InvalidTemplateJson(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(data_json)
            .map_err(|e| ServiceError::InvalidDataJson(e.to_string()))?;
        let layout = self.compute_layout(&template, &data)?;
        serde_json::to_string(&layout).map_err(|e| ServiceError::SerializationFailed(e.to_string()))
    }

    /// Typed layout computation for Rust callers.
    pub fn compute_layout(
        &self,
        template: &Template,
        data: &serde_json::Value,
    ) -> ServiceResult<LayoutResult> {
        let fonts = self.fonts_for_template(&template.fonts);
        dreport_layout::compute_layout(template, data, &fonts)
            .map_err(|e| ServiceError::LayoutFailed(e.to_string()))
    }

    /// Render a PDF from JSON inputs.
    pub fn render_pdf_json(
        &self,
        template_json: &str,
        data_json: &str,
    ) -> ServiceResult<Vec<u8>> {
        let template: Template = serde_json::from_str(template_json)
            .map_err(|e| ServiceError::InvalidTemplateJson(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(data_json)
            .map_err(|e| ServiceError::InvalidDataJson(e.to_string()))?;
        self.render_pdf(&template, &data)
    }

    /// Typed PDF rendering for Rust callers.
    pub fn render_pdf(
        &self,
        template: &Template,
        data: &serde_json::Value,
    ) -> ServiceResult<Vec<u8>> {
        let fonts = self.fonts_for_template(&template.fonts);
        let layout = dreport_layout::compute_layout(template, data, &fonts)
            .map_err(|e| ServiceError::LayoutFailed(e.to_string()))?;
        dreport_layout::pdf_render::render_pdf(&layout, &fonts)
            .map_err(ServiceError::PdfFailed)
    }

    /// Snapshot the FontData set required for the given template families.
    /// Held briefly under read lock then released — the resulting Vec is owned.
    fn fonts_for_template(&self, families: &[String]) -> Vec<FontData> {
        let reg = self.registry.read().expect("font registry poisoned");
        reg.fonts_for_families(families)
    }
}

impl Default for DreportService {
    fn default() -> Self {
        Self::new()
    }
}

/// Allow consumers to use `&DreportService` wherever a `FontProvider` is expected.
impl FontProvider for DreportService {
    fn list_families(&self) -> Vec<FontFamilyInfo> {
        self.list_font_families()
    }

    fn load_font(&self, family: &str, weight: u16, italic: bool) -> Option<FontData> {
        let reg = self.registry.read().expect("font registry poisoned");
        reg.load_font(family, weight, italic)
    }
}
