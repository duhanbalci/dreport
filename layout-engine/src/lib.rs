pub mod sizing;
pub mod text_measure;
pub mod data_resolve;
pub mod table_layout;
pub mod tree;
pub mod page_break;
pub mod expr_eval;

#[cfg(target_arch = "wasm32")]
pub mod wasm_api;

pub mod barcode_gen;
pub mod chart_render;

#[cfg(not(target_arch = "wasm32"))]
pub mod pdf_render;

use dreport_core::models::{ChartType, Template};
use serde::{Deserialize, Serialize};

// --- Layout sonuç tipleri ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub pages: Vec<PageLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayout {
    pub page_index: usize,
    pub width_mm: f64,
    pub height_mm: f64,
    pub elements: Vec<ElementLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLayout {
    pub id: String,
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
    pub element_type: String,
    pub content: Option<ResolvedContent>,
    pub style: ResolvedStyle,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResolvedContent {
    #[serde(rename = "text")]
    Text { value: String },
    #[serde(rename = "image")]
    Image { src: String },
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "barcode")]
    Barcode { format: String, value: String },
    #[serde(rename = "page_number")]
    PageNumber { current: usize, total: usize },
    #[serde(rename = "shape")]
    Shape {
        #[serde(rename = "shapeType")]
        shape_type: String,
    },
    #[serde(rename = "checkbox")]
    Checkbox { checked: bool },
    #[serde(rename = "rich_text")]
    RichText { spans: Vec<ResolvedRichSpan> },
    #[serde(rename = "table")]
    Table {
        headers: Vec<TableHeaderCell>,
        rows: Vec<Vec<TableCell>>,
        column_widths_mm: Vec<f64>,
    },
    #[serde(rename = "chart")]
    Chart {
        svg: String,
        /// PDF render icin chart verisi (frontend bunu kullanmaz)
        #[serde(flatten)]
        chart_data: ChartRenderData,
    },
}

/// PDF renderer icin chart verisi — ResolvedContent::Chart icinde tasınır
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartRenderData {
    pub chart_type: ChartType,
    pub categories: Vec<String>,
    pub series: Vec<ChartSeriesData>,
    #[serde(default)]
    pub title_text: Option<String>,
    #[serde(default)]
    pub title_font_size: Option<f64>,
    #[serde(default)]
    pub title_color: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub show_labels: bool,
    #[serde(default)]
    pub label_font_size: Option<f64>,
    #[serde(default)]
    pub show_grid: bool,
    #[serde(default)]
    pub grid_color: Option<String>,
    #[serde(default)]
    pub bar_gap: Option<f64>,
    #[serde(default)]
    pub stacked: bool,
    #[serde(default)]
    pub inner_radius: Option<f64>,
    #[serde(default)]
    pub show_points: Option<bool>,
    #[serde(default)]
    pub line_width: Option<f64>,
    #[serde(default)]
    pub background_color: Option<String>,
    // Label color
    #[serde(default)]
    pub label_color: Option<String>,
    // Legend
    #[serde(default)]
    pub legend_show: bool,
    #[serde(default)]
    pub legend_position: Option<String>,
    #[serde(default)]
    pub legend_font_size: Option<f64>,
    // Axis labels
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
    // Title align
    #[serde(default)]
    pub title_align: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeriesData {
    pub name: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRichSpan {
    pub text: String,
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeaderCell {
    pub text: String,
    pub align: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub text: String,
    pub align: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedStyle {
    // Text
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub color: Option<String>,
    pub text_align: Option<String>,
    // Line
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
    // Container
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f64>,
    pub border_radius: Option<f64>,
    pub border_style: Option<String>,
    // Table
    pub header_bg: Option<String>,
    pub header_color: Option<String>,
    pub zebra_odd: Option<String>,
    pub zebra_even: Option<String>,
    pub header_font_size: Option<f64>,
    // Image
    pub object_fit: Option<String>,
    // Barcode
    pub barcode_color: Option<String>,
    pub barcode_include_text: Option<bool>,
}

/// Ana layout hesaplama fonksiyonu.
/// Template + data + font verileri alır, her element için pozisyon döner.
pub fn compute_layout(
    template: &Template,
    data: &serde_json::Value,
    font_data: &[FontData],
) -> LayoutResult {
    let mut measurer = text_measure::TextMeasurer::new(font_data);
    let resolved = data_resolve::resolve_template(template, data);
    tree::compute(template, &resolved, &mut measurer)
}

/// Cache-aware layout hesaplama.
/// Önceki çağrıdan kalan text measurement cache'ini alır, hesaplama sonrası
/// güncellenen cache'i geri döner. WASM tarafında cross-call persist için kullanılır.
pub fn compute_layout_cached(
    template: &Template,
    data: &serde_json::Value,
    font_data: &[FontData],
    text_cache: text_measure::TextMeasureCache,
) -> (LayoutResult, text_measure::TextMeasureCache) {
    let mut measurer = text_measure::TextMeasurer::new_with_cache(font_data, text_cache);
    let resolved = data_resolve::resolve_template(template, data);
    let result = tree::compute(template, &resolved, &mut measurer);
    (result, measurer.take_cache())
}

/// Font verisi (ham TTF/OTF bytes)
#[derive(Debug, Clone)]
pub struct FontData {
    pub family: String,
    pub data: Vec<u8>,
}
