//! Shared chart layout computation — used by both SVG (chart_render) and PDF (pdf_render).
//!
//! This module extracts the **what to draw and where** logic into shared structs.
//! Each renderer then handles the **how** (actual drawing calls) using these structs.

use dreport_core::models::ChartType;

pub const DEFAULT_COLORS: &[&str] = &[
    "#4F46E5", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
];

// ---------------------------------------------------------------------------
// Shared structs
// ---------------------------------------------------------------------------

pub struct ChartLayout {
    /// Absolute plot area origin X (mm). For SVG this equals margin_left;
    /// for PDF this is base_x_mm + margin_left.
    pub plot_x: f64,
    /// Absolute plot area origin Y (mm).
    pub plot_y: f64,
    pub plot_w: f64,
    pub plot_h: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub palette: Vec<String>,
    pub title: Option<TitleLayout>,
    pub legend_show: bool,
    pub legend_pos: String,
    pub legend_font: f64,
}

pub struct TitleLayout {
    pub text: String,
    pub font_size: f64,
    pub color: String,
    /// x position in mm (absolute)
    pub x: f64,
    /// y position in mm (absolute)
    pub y: f64,
    pub align: String, // "left", "center", "right"
}

pub struct YAxisLayout {
    pub ticks: Vec<YTick>,
    pub show_grid: bool,
    pub grid_color: String,
    /// Y axis vertical line positions (mm, absolute)
    pub axis_x: f64,
    pub axis_y_start: f64,
    pub axis_y_end: f64,
    /// Right edge of the grid lines (axis_x + plot_w)
    pub grid_end_x: f64,
}

pub struct YTick {
    pub value: f64,
    pub label: String,
    /// Absolute Y position (mm)
    pub y: f64,
}

pub struct XLabelLayout {
    pub labels: Vec<XLabel>,
    /// Rotation angle in degrees (0 = horizontal, 90 = fully vertical).
    /// Dynamically computed based on available space vs label length.
    pub rotate_angle: f64,
}

pub struct XLabel {
    pub text: String,
    /// Absolute X position (mm)
    pub x: f64,
    /// Absolute Y position (mm)
    pub y: f64,
}

/// Pre-computed bar geometry for a single bar rect
pub struct BarRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub color_idx: usize,
    pub value: f64,
    /// Label position (center x, label y)
    pub label_x: f64,
    pub label_y: f64,
}

pub struct BarChartLayout {
    pub min_val: f64,
    pub max_val: f64,
    pub y_axis: YAxisLayout,
    pub x_labels: XLabelLayout,
    pub bars: Vec<BarRect>,
    pub show_labels: bool,
    pub label_font: f64,
    pub label_color: String,
    pub stacked: bool,
    /// X axis line endpoints
    pub x_axis_y: f64,
    pub x_axis_x1: f64,
    pub x_axis_x2: f64,
}

/// Pre-computed point position for line chart
pub struct LinePoint {
    pub x: f64,
    pub y: f64,
    pub value: f64,
}

pub struct LineSeriesLayout {
    pub color_idx: usize,
    pub points: Vec<LinePoint>,
}

pub struct LineChartLayout {
    pub min_val: f64,
    pub max_val: f64,
    pub y_axis: YAxisLayout,
    pub x_labels: XLabelLayout,
    pub series: Vec<LineSeriesLayout>,
    pub line_width: f64,
    pub show_points: bool,
    pub show_labels: bool,
    pub label_font: f64,
    pub label_color: String,
    pub smooth: bool,
    /// X axis line endpoints
    pub x_axis_y: f64,
    pub x_axis_x1: f64,
    pub x_axis_x2: f64,
    /// Vertical reference lines
    pub ref_lines: Vec<RefLineLayout>,
}

pub struct RefLineLayout {
    pub x: f64,
    pub y1: f64,
    pub y2: f64,
    pub color: String,
    pub width: f64,
    pub dash: bool,
    pub label: Option<String>,
}

pub struct PieSlice {
    pub start_angle: f64,
    pub end_angle: f64,
    pub sweep: f64,
    pub color_idx: usize,
    pub value: f64,
    pub fraction: f64,
    /// Label position inside slice
    pub label_x: f64,
    pub label_y: f64,
    pub label_text: String,
    /// Leader line + category label outside
    pub leader_start_x: f64,
    pub leader_start_y: f64,
    pub leader_end_x: f64,
    pub leader_end_y: f64,
    pub cat_label_x: f64,
    pub cat_label_y: f64,
    pub cat_label_text: String,
    pub cat_label_anchor_end: bool, // true = end/right, false = start/left
}

pub struct PieChartLayout {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub inner_radius: f64,
    pub slices: Vec<PieSlice>,
    pub show_labels: bool,
    /// Category name labels + leader lines outside slices
    pub show_cat_labels: bool,
    pub label_font: f64,
    pub label_color: String,
}

/// Legend item with pre-computed position
pub struct LegendItemLayout {
    pub name: String,
    pub color_idx: usize,
    /// Swatch rect position (mm)
    pub swatch_x: f64,
    pub swatch_y: f64,
    /// Text position (mm)
    pub text_x: f64,
    pub text_y: f64,
}

pub struct LegendLayout {
    pub items: Vec<LegendItemLayout>,
    pub font_size: f64,
    pub position: String,
    pub swatch_size: f64,
}

// ---------------------------------------------------------------------------
// Common input abstraction — both ResolvedChartData and ChartRenderData
// can provide these values
// ---------------------------------------------------------------------------

/// Trait that abstracts over the two chart data representations used by
/// SVG renderer (ResolvedChartData) and PDF renderer (ChartRenderData).
pub trait ChartDataSource {
    fn chart_type(&self) -> ChartType;
    fn categories(&self) -> &[String];
    fn series_count(&self) -> usize;
    fn series_name(&self, idx: usize) -> &str;
    fn series_values(&self, idx: usize) -> &[f64];
    fn title_text(&self) -> Option<&str>;
    fn title_font_size(&self) -> Option<f64>;
    fn title_color(&self) -> Option<&str>;
    fn title_align(&self) -> Option<&str>;
    fn legend_show(&self) -> bool;
    fn legend_position(&self) -> Option<&str>;
    fn legend_font_size(&self) -> Option<f64>;
    fn x_label(&self) -> Option<&str>;
    fn y_label(&self) -> Option<&str>;
    fn show_grid(&self) -> bool;
    fn grid_color(&self) -> Option<&str>;
    fn bar_gap(&self) -> Option<f64>;
    fn stacked(&self) -> bool;
    fn colors(&self) -> Option<&[String]>;
    fn background_color(&self) -> Option<&str>;
    fn show_labels(&self) -> bool;
    fn label_font_size(&self) -> Option<f64>;
    fn label_color(&self) -> Option<&str>;
    fn inner_radius(&self) -> Option<f64>;
    fn show_points(&self) -> Option<bool>;
    fn line_width(&self) -> Option<f64>;
    fn curve_type(&self) -> Option<&str>;
    fn reference_lines(&self) -> &[dreport_core::models::ChartReferenceLine];
    fn show_vertical_grid(&self) -> bool;
    fn vertical_grid_color(&self) -> Option<&str>;
}

// ---------------------------------------------------------------------------
// Impl for SVG renderer's ResolvedChartData
// ---------------------------------------------------------------------------

impl ChartDataSource for crate::data_resolve::ResolvedChartData {
    fn chart_type(&self) -> ChartType {
        self.chart_type.clone()
    }
    fn categories(&self) -> &[String] {
        &self.categories
    }
    fn series_count(&self) -> usize {
        self.series.len()
    }
    fn series_name(&self, idx: usize) -> &str {
        &self.series[idx].name
    }
    fn series_values(&self, idx: usize) -> &[f64] {
        &self.series[idx].values
    }
    fn title_text(&self) -> Option<&str> {
        self.title
            .as_ref()
            .map(|t| t.text.as_str())
            .filter(|t| !t.is_empty())
    }
    fn title_font_size(&self) -> Option<f64> {
        self.title.as_ref().and_then(|t| t.font_size)
    }
    fn title_color(&self) -> Option<&str> {
        self.title.as_ref().and_then(|t| t.color.as_deref())
    }
    fn title_align(&self) -> Option<&str> {
        self.title.as_ref().and_then(|t| t.align.as_deref())
    }
    fn legend_show(&self) -> bool {
        self.legend.as_ref().is_some_and(|l| l.show)
    }
    fn legend_position(&self) -> Option<&str> {
        self.legend.as_ref().and_then(|l| l.position.as_deref())
    }
    fn legend_font_size(&self) -> Option<f64> {
        self.legend.as_ref().and_then(|l| l.font_size)
    }
    fn x_label(&self) -> Option<&str> {
        self.axis.as_ref().and_then(|a| a.x_label.as_deref())
    }
    fn y_label(&self) -> Option<&str> {
        self.axis.as_ref().and_then(|a| a.y_label.as_deref())
    }
    fn show_grid(&self) -> bool {
        self.axis.as_ref().and_then(|a| a.show_grid).unwrap_or(true)
    }
    fn grid_color(&self) -> Option<&str> {
        self.axis.as_ref().and_then(|a| a.grid_color.as_deref())
    }
    fn bar_gap(&self) -> Option<f64> {
        self.style.bar_gap
    }
    fn stacked(&self) -> bool {
        matches!(
            self.group_mode,
            Some(dreport_core::models::GroupMode::Stacked)
        )
    }
    fn colors(&self) -> Option<&[String]> {
        self.style.colors.as_deref()
    }
    fn background_color(&self) -> Option<&str> {
        self.style.background_color.as_deref()
    }
    fn show_labels(&self) -> bool {
        self.labels.as_ref().is_some_and(|l| l.show)
    }
    fn label_font_size(&self) -> Option<f64> {
        self.labels.as_ref().and_then(|l| l.font_size)
    }
    fn label_color(&self) -> Option<&str> {
        self.labels.as_ref().and_then(|l| l.color.as_deref())
    }
    fn inner_radius(&self) -> Option<f64> {
        self.style.inner_radius
    }
    fn show_points(&self) -> Option<bool> {
        self.style.show_points
    }
    fn line_width(&self) -> Option<f64> {
        self.style.line_width
    }
    fn curve_type(&self) -> Option<&str> {
        self.style.curve_type.as_deref()
    }
    fn reference_lines(&self) -> &[dreport_core::models::ChartReferenceLine] {
        self.axis.as_ref().map_or(&[], |a| &a.reference_lines)
    }
    fn show_vertical_grid(&self) -> bool {
        self.axis.as_ref().and_then(|a| a.show_vertical_grid).unwrap_or(true)
    }
    fn vertical_grid_color(&self) -> Option<&str> {
        self.axis.as_ref().and_then(|a| a.vertical_grid_color.as_deref())
    }
}

// ---------------------------------------------------------------------------
// Impl for PDF renderer's ChartRenderData
// ---------------------------------------------------------------------------

impl ChartDataSource for crate::ChartRenderData {
    fn chart_type(&self) -> ChartType {
        self.chart_type.clone()
    }
    fn categories(&self) -> &[String] {
        &self.categories
    }
    fn series_count(&self) -> usize {
        self.series.len()
    }
    fn series_name(&self, idx: usize) -> &str {
        &self.series[idx].name
    }
    fn series_values(&self, idx: usize) -> &[f64] {
        &self.series[idx].values
    }
    fn title_text(&self) -> Option<&str> {
        self.title_text.as_deref().filter(|t| !t.is_empty())
    }
    fn title_font_size(&self) -> Option<f64> {
        self.title_font_size
    }
    fn title_color(&self) -> Option<&str> {
        self.title_color.as_deref()
    }
    fn title_align(&self) -> Option<&str> {
        self.title_align.as_deref()
    }
    fn legend_show(&self) -> bool {
        self.legend_show
    }
    fn legend_position(&self) -> Option<&str> {
        self.legend_position.as_deref()
    }
    fn legend_font_size(&self) -> Option<f64> {
        self.legend_font_size
    }
    fn x_label(&self) -> Option<&str> {
        self.x_label.as_deref()
    }
    fn y_label(&self) -> Option<&str> {
        self.y_label.as_deref()
    }
    fn show_grid(&self) -> bool {
        self.show_grid
    }
    fn grid_color(&self) -> Option<&str> {
        self.grid_color.as_deref()
    }
    fn bar_gap(&self) -> Option<f64> {
        self.bar_gap
    }
    fn stacked(&self) -> bool {
        self.stacked
    }
    fn colors(&self) -> Option<&[String]> {
        if self.colors.is_empty() {
            None
        } else {
            Some(&self.colors)
        }
    }
    fn background_color(&self) -> Option<&str> {
        self.background_color.as_deref()
    }
    fn show_labels(&self) -> bool {
        self.show_labels
    }
    fn label_font_size(&self) -> Option<f64> {
        self.label_font_size
    }
    fn label_color(&self) -> Option<&str> {
        self.label_color.as_deref()
    }
    fn inner_radius(&self) -> Option<f64> {
        self.inner_radius
    }
    fn show_points(&self) -> Option<bool> {
        self.show_points
    }
    fn line_width(&self) -> Option<f64> {
        self.line_width
    }
    fn curve_type(&self) -> Option<&str> {
        self.curve_type.as_deref()
    }
    fn reference_lines(&self) -> &[dreport_core::models::ChartReferenceLine] {
        &self.reference_lines
    }
    fn show_vertical_grid(&self) -> bool {
        self.show_vertical_grid
    }
    fn vertical_grid_color(&self) -> Option<&str> {
        self.vertical_grid_color.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Shared computation functions
// ---------------------------------------------------------------------------

pub fn color_at(palette: &[String], i: usize) -> &str {
    &palette[i % palette.len()]
}

pub fn build_palette(data: &dyn ChartDataSource) -> Vec<String> {
    let n_colors = data.categories().len().max(data.series_count()).max(1);
    let user_colors = data.colors();
    (0..n_colors)
        .map(|i| {
            if let Some(uc) = user_colors
                && i < uc.len()
            {
                return uc[i].clone();
            }
            DEFAULT_COLORS[i % DEFAULT_COLORS.len()].to_string()
        })
        .collect()
}

pub fn format_value(v: f64) -> String {
    if v.abs() >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v.abs() >= 1_000.0 {
        format!("{:.1}K", v / 1_000.0)
    } else if v.fract().abs() < 1e-10 {
        format!("{}", v as i64)
    } else {
        format!("{:.1}", v)
    }
}

/// Compute the value range (min, max) across all series.
pub fn compute_value_range(data: &dyn ChartDataSource, stacked: bool) -> (f64, f64) {
    if data.series_count() == 0 {
        return (0.0, 1.0);
    }
    if stacked {
        let n = data.categories().len();
        let mut max_stack = 0.0_f64;
        for ci in 0..n {
            let sum: f64 = (0..data.series_count())
                .map(|si| data.series_values(si).get(ci).copied().unwrap_or(0.0))
                .sum();
            max_stack = max_stack.max(sum);
        }
        (0.0, max_stack * 1.05)
    } else {
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;
        for si in 0..data.series_count() {
            for val in data.series_values(si) {
                min_v = min_v.min(*val);
                max_v = max_v.max(*val);
            }
        }
        if min_v > 0.0 {
            min_v = 0.0;
        }
        max_v *= 1.05;
        (min_v, max_v)
    }
}

fn safe_range(min_val: f64, max_val: f64) -> f64 {
    let r = max_val - min_val;
    if r.abs() < 1e-10 { 1.0 } else { r }
}

/// Compute margins and plot area. `origin_x/y` is 0 for SVG or base_x_mm/base_y_mm for PDF.
pub fn compute_chart_layout(
    data: &dyn ChartDataSource,
    width_mm: f64,
    height_mm: f64,
    origin_x: f64,
    origin_y: f64,
) -> ChartLayout {
    let palette = build_palette(data);

    let mut margin_top = 2.0_f64;
    let mut margin_bottom = 4.0_f64;
    let mut margin_left = 8.0_f64;
    let margin_right = 4.0_f64;

    // Title
    let title = if let Some(text) = data.title_text() {
        let fs = data.title_font_size().unwrap_or(4.0);
        margin_top += fs * 0.4 + 2.0;
        let color = data.title_color().unwrap_or("#333333").to_string();
        let align = data.title_align().unwrap_or("center").to_string();
        let x = match align.as_str() {
            "left" => origin_x + margin_left,
            "right" => origin_x + width_mm - margin_right,
            _ => origin_x + width_mm / 2.0,
        };
        let y = origin_y + margin_top - 1.0;
        Some(TitleLayout {
            text: text.to_string(),
            font_size: fs,
            color,
            x,
            y,
            align,
        })
    } else {
        None
    };

    // Legend space
    let legend_show = data.legend_show();
    let legend_pos = data.legend_position().unwrap_or("bottom").to_string();
    let legend_font = data.legend_font_size().unwrap_or(2.8);

    if legend_show && data.series_count() > 1 {
        match legend_pos.as_str() {
            "top" => margin_top += legend_font + 3.0,
            "bottom" => margin_bottom += legend_font + 3.0,
            _ => {}
        }
    }

    // Axis labels space (bar and line only)
    let has_axis = !matches!(data.chart_type(), ChartType::Pie);
    if has_axis {
        if data.x_label().is_some() {
            margin_bottom += 4.0;
        }
        if data.y_label().is_some() {
            margin_left += 4.0;
        }
        // Category labels bottom space
        let max_label_len = data.categories().iter().map(|c| c.len()).max().unwrap_or(0);
        let n_cats = data.categories().len();
        let available_w = width_mm - margin_left - margin_right;
        let cat_width = if n_cats > 0 {
            available_w / n_cats as f64
        } else {
            available_w
        };
        let rotate_angle = compute_label_rotation(max_label_len, cat_width);
        if rotate_angle > 0.0 {
            let char_w_mm = 2.5 * 0.6;
            let max_text_w = max_label_len as f64 * char_w_mm;
            let angle_rad = rotate_angle.to_radians();
            let label_v = max_text_w * angle_rad.sin();
            margin_bottom += label_v.clamp(6.0, 25.0);
            let label_h = max_text_w * angle_rad.cos();
            let extra_left = (label_h - cat_width / 2.0).max(0.0);
            margin_left += extra_left.min(10.0);
        } else {
            margin_bottom += 4.0;
        }
        // Y-axis value labels left space
        margin_left += 6.0;
    }

    let plot_x = origin_x + margin_left;
    let plot_y = origin_y + margin_top;
    let plot_w = (width_mm - margin_left - margin_right).max(1.0);
    let plot_h = (height_mm - margin_top - margin_bottom).max(1.0);

    ChartLayout {
        plot_x,
        plot_y,
        plot_w,
        plot_h,
        margin_top,
        margin_bottom,
        margin_left,
        margin_right,
        palette,
        title,
        legend_show,
        legend_pos,
        legend_font,
    }
}

/// Compute Y axis ticks and grid lines.
#[allow(clippy::too_many_arguments)]
pub fn compute_y_axis(
    min_val: f64,
    max_val: f64,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    show_grid: bool,
    grid_color: &str,
) -> YAxisLayout {
    let range = safe_range(min_val, max_val);
    let tick_count = 5;
    let ticks = (0..=tick_count)
        .map(|i| {
            let frac = i as f64 / tick_count as f64;
            let val = min_val + frac * range;
            let y = py + ph - frac * ph;
            YTick {
                value: val,
                label: format_value(val),
                y,
            }
        })
        .collect();

    YAxisLayout {
        ticks,
        show_grid,
        grid_color: grid_color.to_string(),
        axis_x: px,
        axis_y_start: py,
        axis_y_end: py + ph,
        grid_end_x: px + pw,
    }
}

/// Compute dynamic label rotation angle (degrees) based on available space.
/// Uses Chart.js-style algorithm: rotate only when labels overflow their slot,
/// and use the minimum angle that prevents overlap.
fn compute_label_rotation(max_label_len: usize, slot_width: f64) -> f64 {
    let label_font_size = 2.5_f64;
    let char_w_mm = label_font_size * 0.6;
    let max_label_w = max_label_len as f64 * char_w_mm;
    let padding = label_font_size * 0.5;

    // Labels fit horizontally — no rotation needed
    if (max_label_w + padding) <= slot_width {
        return 0.0;
    }

    // Chart.js Constraint A: sin(angle) = (label_height + padding) / slot_width
    // This finds the minimum angle where the rotated label's projected height
    // fits within the tick slot width, preventing horizontal overlap.
    let label_h = label_font_size;
    let sin_val = ((label_h + padding) / slot_width).clamp(0.0, 1.0);
    let angle_deg = sin_val.asin().to_degrees();
    angle_deg.clamp(0.0, 50.0)
}

/// Compute X label positions for bar chart (slot-based spacing).
pub fn compute_x_labels_bar(
    categories: &[String],
    px: f64,
    baseline_y: f64,
    pw: f64,
) -> XLabelLayout {
    let n_cats = categories.len();
    if n_cats == 0 {
        return XLabelLayout {
            labels: vec![],
            rotate_angle: 0.0,
        };
    }
    let cat_width = pw / n_cats as f64;
    let max_label_len = categories.iter().map(|c| c.len()).max().unwrap_or(0);
    let rotate_angle = compute_label_rotation(max_label_len, cat_width);
    let labels = categories
        .iter()
        .enumerate()
        .map(|(ci, cat)| XLabel {
            text: cat.clone(),
            x: px + ci as f64 * cat_width + cat_width / 2.0,
            y: baseline_y + 2.5,
        })
        .collect();
    XLabelLayout {
        labels,
        rotate_angle,
    }
}

/// Compute X label positions for line chart (point-based spacing).
pub fn compute_x_labels_line(
    categories: &[String],
    px: f64,
    baseline_y: f64,
    pw: f64,
) -> XLabelLayout {
    let n_cats = categories.len();
    if n_cats == 0 {
        return XLabelLayout {
            labels: vec![],
            rotate_angle: 0.0,
        };
    }
    let step = pw / n_cats as f64;
    let max_label_len = categories.iter().map(|c| c.len()).max().unwrap_or(0);
    let rotate_angle = compute_label_rotation(max_label_len, step);
    let labels = categories
        .iter()
        .enumerate()
        .map(|(ci, cat)| {
            let x = px + step / 2.0 + ci as f64 * step;
            XLabel {
                text: cat.clone(),
                x,
                y: baseline_y + 2.5,
            }
        })
        .collect();
    XLabelLayout {
        labels,
        rotate_angle,
    }
}

/// Compute bar chart layout (all bar geometries + axes).
pub fn compute_bar_layout(data: &dyn ChartDataSource, cl: &ChartLayout) -> BarChartLayout {
    let px = cl.plot_x;
    let py = cl.plot_y;
    let pw = cl.plot_w;
    let ph = cl.plot_h;

    let stacked = data.stacked();
    let (min_val, max_val) = compute_value_range(data, stacked);
    let range = safe_range(min_val, max_val);

    let show_grid = data.show_grid();
    let grid_color = data.grid_color().unwrap_or("#E5E7EB");
    let y_axis = compute_y_axis(min_val, max_val, px, py, pw, ph, show_grid, grid_color);

    let n_cats = data.categories().len();
    let n_series = data.series_count();
    let cat_width = if n_cats > 0 { pw / n_cats as f64 } else { pw };
    let bar_gap = data.bar_gap().unwrap_or(0.2).clamp(0.0, 0.8);
    let group_width = cat_width * (1.0 - bar_gap);

    let show_labels = data.show_labels();
    let label_font = data.label_font_size().unwrap_or(2.2);
    let label_color = data.label_color().unwrap_or("#333").to_string();

    let mut bars = Vec::new();

    for ci in 0..n_cats {
        let cat_x = px + ci as f64 * cat_width;
        if stacked {
            let mut y_offset = 0.0_f64;
            for si in 0..n_series {
                let val = data.series_values(si).get(ci).copied().unwrap_or(0.0);
                let bar_h = (val / range) * ph;
                let bar_y = py + ph - y_offset - bar_h;
                let bx = cat_x + cat_width * bar_gap / 2.0;
                bars.push(BarRect {
                    x: bx,
                    y: bar_y,
                    w: group_width,
                    h: bar_h.max(0.0),
                    color_idx: si,
                    value: val,
                    label_x: cat_x + cat_width / 2.0,
                    label_y: bar_y + bar_h / 2.0 + label_font * 0.15,
                });
                y_offset += bar_h;
            }
        } else {
            let bar_w = if n_series > 0 {
                group_width / n_series as f64
            } else {
                group_width
            };
            for si in 0..n_series {
                let val = data.series_values(si).get(ci).copied().unwrap_or(0.0);
                let bar_h = ((val - min_val) / range) * ph;
                let bx = cat_x + cat_width * bar_gap / 2.0 + si as f64 * bar_w;
                let by = py + ph - bar_h;
                bars.push(BarRect {
                    x: bx,
                    y: by,
                    w: bar_w.max(0.1),
                    h: bar_h.max(0.0),
                    color_idx: si,
                    value: val,
                    label_x: bx + bar_w / 2.0,
                    label_y: by - 0.8,
                });
            }
        }
    }

    let x_labels = compute_x_labels_bar(data.categories(), px, py + ph, pw);

    BarChartLayout {
        min_val,
        max_val,
        y_axis,
        x_labels,
        bars,
        show_labels,
        label_font,
        label_color,
        stacked,
        x_axis_y: py + ph,
        x_axis_x1: px,
        x_axis_x2: px + pw,
    }
}

/// Compute line chart layout (all point positions + axes).
pub fn compute_line_layout(data: &dyn ChartDataSource, cl: &ChartLayout) -> LineChartLayout {
    let px = cl.plot_x;
    let py = cl.plot_y;
    let pw = cl.plot_w;
    let ph = cl.plot_h;

    let (min_val, max_val) = compute_value_range(data, false);
    let range = safe_range(min_val, max_val);
    let n_cats = data.categories().len();

    let show_grid = data.show_grid();
    let grid_color = data.grid_color().unwrap_or("#E5E7EB");
    let y_axis = compute_y_axis(min_val, max_val, px, py, pw, ph, show_grid, grid_color);

    let line_width = data.line_width().unwrap_or(0.5);
    let show_points = data.show_points().unwrap_or(true);
    let show_labels = data.show_labels();
    let label_font = data.label_font_size().unwrap_or(2.2);
    let label_color = data.label_color().unwrap_or("#333").to_string();
    let smooth = data.curve_type() == Some("smooth");

    // Slot-based positioning: each category gets a slot, point centered in slot
    // This adds padding on left/right so first/last points don't touch axes
    let step = if n_cats > 0 { pw / n_cats as f64 } else { pw };

    let series = (0..data.series_count())
        .map(|si| {
            let values = data.series_values(si);
            let points = values
                .iter()
                .enumerate()
                .map(|(ci, val)| {
                    let x = px + step / 2.0 + ci as f64 * step;
                    let y = py + ph - ((val - min_val) / range) * ph;
                    LinePoint { x, y, value: *val }
                })
                .collect();
            LineSeriesLayout {
                color_idx: si,
                points,
            }
        })
        .collect();

    let x_labels = compute_x_labels_line(data.categories(), px, py + ph, pw);

    // Vertical grid lines at each category
    let vgrid_color = data.vertical_grid_color().unwrap_or("#E5E7EB").to_string();
    let mut ref_lines: Vec<RefLineLayout> = if data.show_vertical_grid() {
        (0..n_cats).map(|ci| {
            let x = px + step / 2.0 + ci as f64 * step;
            RefLineLayout {
                x,
                y1: py,
                y2: py + ph,
                color: vgrid_color.clone(),
                width: 0.15,
                dash: false,
                label: None,
            }
        }).collect()
    } else {
        vec![]
    };

    // Explicit reference lines (overlay on top of grid)
    for rl in data.reference_lines() {
        if rl.category_index >= n_cats {
            continue;
        }
        let x = px + step / 2.0 + rl.category_index as f64 * step;
        ref_lines.push(RefLineLayout {
            x,
            y1: py,
            y2: py + ph,
            color: rl.color.clone().unwrap_or_else(|| "#9CA3AF".to_string()),
            width: rl.width.unwrap_or(0.3),
            dash: rl.dash.unwrap_or(true),
            label: rl.label.clone(),
        });
    }

    LineChartLayout {
        min_val,
        max_val,
        y_axis,
        x_labels,
        series,
        line_width,
        show_points,
        show_labels,
        label_font,
        label_color,
        smooth,
        x_axis_y: py + ph,
        x_axis_x1: px,
        x_axis_x2: px + pw,
        ref_lines,
    }
}

/// Compute pie chart layout (slice angles and label positions).
pub fn compute_pie_layout(data: &dyn ChartDataSource, cl: &ChartLayout) -> PieChartLayout {
    let px = cl.plot_x;
    let py = cl.plot_y;
    let pw = cl.plot_w;
    let ph = cl.plot_h;

    let values: Vec<f64> = if data.series_count() == 1 {
        data.series_values(0).to_vec()
    } else {
        data.categories()
            .iter()
            .enumerate()
            .map(|(ci, _)| {
                (0..data.series_count())
                    .map(|si| data.series_values(si).get(ci).copied().unwrap_or(0.0))
                    .sum()
            })
            .collect()
    };

    let total: f64 = values.iter().sum();
    let show_labels = data.show_labels();
    let label_font = data.label_font_size().unwrap_or(3.0);
    let label_color = data.label_color().unwrap_or("#333").to_string();

    let cx = px + pw / 2.0;
    let cy = py + ph / 2.0;
    let radius = pw.min(ph) / 2.0 * 0.65;
    let inner_frac = data.inner_radius().unwrap_or(0.0).clamp(0.0, 0.9);
    let inner_r = radius * inner_frac;

    let mut slices = Vec::new();

    if total > 0.0 {
        let mut start_angle = -std::f64::consts::FRAC_PI_2;
        let categories = data.categories();

        for (i, val) in values.iter().enumerate() {
            if *val <= 0.0 {
                start_angle += 0.0; // skip
                continue;
            }
            let sweep = (val / total) * std::f64::consts::TAU;
            let end_angle = start_angle + sweep;
            let mid_angle = start_angle + sweep / 2.0;

            // Label inside slice
            let label_r = if inner_r > 0.0 {
                (radius + inner_r) / 2.0
            } else {
                radius * 0.65
            };
            let lx = cx + label_r * mid_angle.cos();
            let ly = cy + label_r * mid_angle.sin();
            let pct = (val / total * 100.0).round();

            // Leader line + category label
            let line_start_r = radius;
            let line_end_r = radius + 3.0;
            let text_r = radius + 4.0;

            let leader_sx = cx + line_start_r * mid_angle.cos();
            let leader_sy = cy + line_start_r * mid_angle.sin();
            let leader_ex = cx + line_end_r * mid_angle.cos();
            let leader_ey = cy + line_end_r * mid_angle.sin();
            let cat_lx = cx + text_r * mid_angle.cos();
            let cat_ly = cy + text_r * mid_angle.sin();
            let cat_text = if i < categories.len() {
                categories[i].clone()
            } else {
                String::new()
            };
            let anchor_end = mid_angle.cos() < 0.0;

            slices.push(PieSlice {
                start_angle,
                end_angle,
                sweep,
                color_idx: i,
                value: *val,
                fraction: val / total,
                label_x: lx,
                label_y: ly,
                label_text: format!("{}%", pct),
                leader_start_x: leader_sx,
                leader_start_y: leader_sy,
                leader_end_x: leader_ex,
                leader_end_y: leader_ey,
                cat_label_x: cat_lx,
                cat_label_y: cat_ly,
                cat_label_text: cat_text,
                cat_label_anchor_end: anchor_end,
            });

            start_angle = end_angle;
        }
    }

    PieChartLayout {
        cx,
        cy,
        radius,
        inner_radius: inner_r,
        slices,
        show_labels,
        show_cat_labels: show_labels,
        label_font,
        label_color,
    }
}

/// Compute legend item positions.
pub fn compute_legend(
    data: &dyn ChartDataSource,
    cl: &ChartLayout,
    origin_x: f64,
    origin_y: f64,
    total_w: f64,
    total_h: f64,
) -> LegendLayout {
    let font_size = cl.legend_font;
    let position = cl.legend_pos.clone();
    let swatch_size = 2.5;
    let item_gap = 3.0 + font_size * 0.4;
    let spacing = 4.0;

    let is_pie = matches!(data.chart_type(), ChartType::Pie);
    let names: Vec<String> = if is_pie {
        data.categories().to_vec()
    } else {
        (0..data.series_count())
            .map(|i| data.series_name(i).to_string())
            .collect()
    };

    let mut items = Vec::new();

    match position.as_str() {
        "top" => {
            let y = origin_y + cl.margin_top - font_size - 1.5;
            let mut x = origin_x + cl.margin_left;
            for (i, name) in names.iter().enumerate() {
                items.push(LegendItemLayout {
                    name: name.clone(),
                    color_idx: i,
                    swatch_x: x,
                    swatch_y: y - font_size * 0.3,
                    text_x: x + item_gap,
                    text_y: y + font_size * 0.3,
                });
                x += item_gap + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
        "right" => {
            let x = origin_x + cl.margin_left + cl.plot_w + 4.0;
            let mut y = origin_y + cl.margin_top + 2.0;
            for (i, name) in names.iter().enumerate() {
                items.push(LegendItemLayout {
                    name: name.clone(),
                    color_idx: i,
                    swatch_x: x,
                    swatch_y: y,
                    text_x: x + item_gap,
                    text_y: y + font_size * 0.7,
                });
                y += font_size + 2.0;
            }
        }
        _ => {
            // bottom (default)
            let y = origin_y + total_h - 3.0;
            let total_legend_w: f64 = names
                .iter()
                .map(|n| item_gap + n.len() as f64 * font_size * 0.5 + spacing)
                .sum::<f64>()
                - spacing;
            let mut x = origin_x + (total_w - total_legend_w) / 2.0;
            for (i, name) in names.iter().enumerate() {
                items.push(LegendItemLayout {
                    name: name.clone(),
                    color_idx: i,
                    swatch_x: x,
                    swatch_y: y - font_size * 0.3,
                    text_x: x + item_gap,
                    text_y: y + font_size * 0.3,
                });
                x += item_gap + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
    }

    LegendLayout {
        items,
        font_size,
        position,
        swatch_size,
    }
}
