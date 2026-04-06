use crate::data_resolve::ResolvedChartData;
use dreport_core::models::{ChartType, GroupMode};
use std::fmt::Write;

pub const DEFAULT_COLORS: &[&str] = &[
    "#4F46E5", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
];

fn color_at(palette: &[String], i: usize) -> &str {
    &palette[i % palette.len()]
}

/// mm cinsinden chart SVG uret
pub fn render_svg(data: &ResolvedChartData, width_mm: f64, height_mm: f64) -> String {
    let mut svg = String::with_capacity(4096);

    let bg = data
        .style
        .background_color
        .as_deref()
        .unwrap_or("#FFFFFF");

    write!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="100%" height="100%">"##,
        width_mm, height_mm
    )
    .unwrap();
    write!(
        svg,
        r##"<rect width="{}" height="{}" fill="{}"/>"##,
        width_mm, height_mm, bg
    )
    .unwrap();

    // Max sayida renk: kategoriler + seriler
    let n_colors = data.categories.len().max(data.series.len()).max(1);
    let palette: Vec<String> = (0..n_colors)
        .map(|i| {
            if let Some(ref user_colors) = data.style.colors {
                if i < user_colors.len() {
                    return user_colors[i].clone();
                }
            }
            DEFAULT_COLORS[i % DEFAULT_COLORS.len()].to_string()
        })
        .collect();
    // Margin hesaplari
    let mut margin_top = 2.0_f64;
    let mut margin_bottom = 4.0_f64;
    let mut margin_left = 8.0_f64;
    let margin_right = 4.0_f64;

    // Title
    if let Some(ref title) = data.title {
        if !title.text.is_empty() {
            let font_size = title.font_size.unwrap_or(4.0);
            margin_top += font_size * 0.4 + 2.0;
            let color = title.color.as_deref().unwrap_or("#333333");
            let align = title.align.as_deref().unwrap_or("center");
            let x = match align {
                "left" => margin_left,
                "right" => width_mm - margin_right,
                _ => width_mm / 2.0,
            };
            let anchor = match align {
                "left" => "start",
                "right" => "end",
                _ => "middle",
            };
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="{}" font-weight="bold">{}</text>"##,
                x,
                margin_top - 1.0,
                font_size,
                color,
                anchor,
                escape_xml(&title.text)
            )
            .unwrap();
        }
    }

    // Legend space
    let legend_show = data.legend.as_ref().is_some_and(|l| l.show);
    let legend_pos = data
        .legend
        .as_ref()
        .and_then(|l| l.position.as_deref())
        .unwrap_or("bottom");
    let legend_font = data
        .legend
        .as_ref()
        .and_then(|l| l.font_size)
        .unwrap_or(2.8);

    if legend_show && data.series.len() > 1 {
        match legend_pos {
            "top" => margin_top += legend_font + 3.0,
            "bottom" => margin_bottom += legend_font + 3.0,
            _ => {} // right — icerde handle edilecek
        }
    }

    // Axis labels icin yer ac (bar ve line)
    let has_axis = !matches!(data.chart_type, ChartType::Pie);
    if has_axis {
        if data.axis.as_ref().and_then(|a| a.x_label.as_ref()).is_some() {
            margin_bottom += 4.0;
        }
        if data.axis.as_ref().and_then(|a| a.y_label.as_ref()).is_some() {
            margin_left += 4.0;
        }
        // Category labels icin alt bosluk
        let max_label_len = data.categories.iter().map(|c| c.len()).max().unwrap_or(0);
        let n_cats = data.categories.len();
        let available_w = width_mm - margin_left - margin_right;
        let cat_width = if n_cats > 0 {
            available_w / n_cats as f64
        } else {
            available_w
        };
        let max_chars_fit = (cat_width / 1.25).max(1.0) as usize;
        let will_rotate = max_label_len > max_chars_fit;
        if will_rotate {
            // Rotated labels (-45°): dikey ≈ text_width * sin(45°), yatay ≈ text_width * cos(45°)
            let char_w_mm = 1.1;
            let max_text_w = max_label_len as f64 * char_w_mm;
            let label_v = max_text_w * 0.707; // sin(45°)
            margin_bottom += label_v.min(25.0).max(6.0);
            // Sol taraftaki label yana tasabilir
            let label_h = max_text_w * 0.707; // cos(45°)
            let extra_left = (label_h - cat_width / 2.0).max(0.0);
            margin_left += extra_left.min(10.0);
        } else {
            margin_bottom += 4.0;
        }
        // Y-axis value labels icin sol bosluk
        margin_left += 6.0;
    }

    let plot_x = margin_left;
    let plot_y = margin_top;
    let plot_w = (width_mm - margin_left - margin_right).max(1.0);
    let plot_h = (height_mm - margin_top - margin_bottom).max(1.0);

    match data.chart_type {
        ChartType::Bar => render_bar(&mut svg, data, &palette, plot_x, plot_y, plot_w, plot_h),
        ChartType::Line => render_line(&mut svg, data, &palette, plot_x, plot_y, plot_w, plot_h),
        ChartType::Pie => render_pie(&mut svg, data, &palette, width_mm, height_mm, plot_x, plot_y, plot_w, plot_h),
    }

    // Legend render
    if legend_show && data.series.len() > 1 {
        render_legend(&mut svg, data, &palette, legend_pos, legend_font, width_mm, height_mm, margin_left, margin_top, plot_w, plot_h);
    }

    // Axis labels
    if has_axis {
        if let Some(ref axis) = data.axis {
            if let Some(ref x_label) = axis.x_label {
                let x = plot_x + plot_w / 2.0;
                let y = height_mm - 2.0;
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="2.8" fill="#666" text-anchor="middle">{}</text>"##,
                    x, y, escape_xml(x_label)
                )
                .unwrap();
            }
            if let Some(ref y_label) = axis.y_label {
                let x = 3.0;
                let y = plot_y + plot_h / 2.0;
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="2.8" fill="#666" text-anchor="middle" transform="rotate(-90,{:.2},{:.2})">{}</text>"##,
                    x, y, x, y, escape_xml(y_label)
                )
                .unwrap();
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

fn render_bar(
    svg: &mut String,
    data: &ResolvedChartData,
    palette: &[String],
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
) {
    if data.categories.is_empty() || data.series.is_empty() {
        return;
    }

    let stacked = matches!(data.group_mode, Some(GroupMode::Stacked));
    let (min_val, max_val) = value_range(data, stacked);

    let show_grid = data.axis.as_ref().and_then(|a| a.show_grid).unwrap_or(true);
    let grid_color = data
        .axis
        .as_ref()
        .and_then(|a| a.grid_color.as_deref())
        .unwrap_or("#E5E7EB");

    // Grid + Y axis labels
    render_y_axis(svg, min_val, max_val, px, py, pw, ph, show_grid, grid_color);

    let n_cats = data.categories.len();
    let n_series = data.series.len();
    let cat_width = pw / n_cats as f64;
    let bar_gap = data.style.bar_gap.unwrap_or(0.2).clamp(0.0, 0.8);
    let group_width = cat_width * (1.0 - bar_gap);

    let show_labels = data.labels.as_ref().is_some_and(|l| l.show);
    let label_font = data.labels.as_ref().and_then(|l| l.font_size).unwrap_or(2.2);
    let label_color = data
        .labels
        .as_ref()
        .and_then(|l| l.color.as_deref())
        .unwrap_or("#333");

    let range = if (max_val - min_val).abs() < 1e-10 {
        1.0
    } else {
        max_val - min_val
    };

    for ci in 0..data.categories.len() {
        let cat_x = px + ci as f64 * cat_width;

        if stacked {
            let mut y_offset = 0.0_f64;
            for (si, series) in data.series.iter().enumerate() {
                let val = series.values.get(ci).copied().unwrap_or(0.0);
                let bar_h = (val / range) * ph;
                let bar_y = py + ph - y_offset - bar_h;
                write!(
                    svg,
                    r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" rx="0.5"/>"##,
                    cat_x + cat_width * bar_gap / 2.0,
                    bar_y,
                    group_width,
                    bar_h.max(0.0),
                    color_at(palette,si)
                )
                .unwrap();
                if show_labels && val > 0.0 {
                    write!(
                        svg,
                        r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                        cat_x + cat_width / 2.0,
                        bar_y + bar_h / 2.0 + label_font * 0.15,
                        label_font,
                        label_color,
                        format_value(val)
                    )
                    .unwrap();
                }
                y_offset += bar_h;
            }
        } else {
            // Grouped
            let bar_w = group_width / n_series as f64;
            for (si, series) in data.series.iter().enumerate() {
                let val = series.values.get(ci).copied().unwrap_or(0.0);
                let bar_h = ((val - min_val) / range) * ph;
                let bar_x = cat_x + cat_width * bar_gap / 2.0 + si as f64 * bar_w;
                let bar_y = py + ph - bar_h;
                write!(
                    svg,
                    r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" rx="0.5"/>"##,
                    bar_x,
                    bar_y,
                    bar_w.max(0.1),
                    bar_h.max(0.0),
                    color_at(palette,si)
                )
                .unwrap();
                if show_labels {
                    write!(
                        svg,
                        r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                        bar_x + bar_w / 2.0,
                        bar_y - 0.8,
                        label_font,
                        label_color,
                        format_value(val)
                    )
                    .unwrap();
                }
            }
        }

    }

    // X axis labels — rotate if too many categories
    render_x_labels(svg, &data.categories, px, py + ph, pw, n_cats);

    // X axis line
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        px, py + ph, px + pw, py + ph
    )
    .unwrap();
}

fn render_line(
    svg: &mut String,
    data: &ResolvedChartData,
    palette: &[String],
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
) {
    if data.categories.is_empty() || data.series.is_empty() {
        return;
    }

    let (min_val, max_val) = value_range(data, false);
    let range = if (max_val - min_val).abs() < 1e-10 {
        1.0
    } else {
        max_val - min_val
    };

    let show_grid = data.axis.as_ref().and_then(|a| a.show_grid).unwrap_or(true);
    let grid_color = data
        .axis
        .as_ref()
        .and_then(|a| a.grid_color.as_deref())
        .unwrap_or("#E5E7EB");
    render_y_axis(svg, min_val, max_val, px, py, pw, ph, show_grid, grid_color);

    let n_cats = data.categories.len();
    let line_w = data.style.line_width.unwrap_or(0.5);
    let show_points = data.style.show_points.unwrap_or(true);
    let show_labels = data.labels.as_ref().is_some_and(|l| l.show);
    let label_font = data.labels.as_ref().and_then(|l| l.font_size).unwrap_or(2.2);
    let label_color = data
        .labels
        .as_ref()
        .and_then(|l| l.color.as_deref())
        .unwrap_or("#333");

    for (si, series) in data.series.iter().enumerate() {
        let color = color_at(palette,si);
        let mut points = String::new();
        let mut point_circles = String::new();

        for (ci, val) in series.values.iter().enumerate() {
            let x = if n_cats == 1 {
                px + pw / 2.0
            } else {
                px + ci as f64 * pw / (n_cats - 1) as f64
            };
            let y = py + ph - ((val - min_val) / range) * ph;
            write!(points, "{:.2},{:.2} ", x, y).unwrap();

            if show_points {
                write!(
                    point_circles,
                    r##"<circle cx="{:.2}" cy="{:.2}" r="0.8" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                    x, y, color
                )
                .unwrap();
            }

            if show_labels {
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                    x, y - 1.5, label_font, label_color, format_value(*val)
                )
                .unwrap();
            }
        }

        write!(
            svg,
            r##"<polyline points="{}" fill="none" stroke="{}" stroke-width="{:.2}" stroke-linejoin="round" stroke-linecap="round"/>"##,
            points.trim(),
            color,
            line_w
        )
        .unwrap();
        svg.push_str(&point_circles);
    }

    // X axis labels — for line chart, spacing is different
    render_x_labels_line(svg, &data.categories, px, py + ph, pw, n_cats);

    // Axis lines
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        px, py + ph, px + pw, py + ph
    )
    .unwrap();
}

fn render_pie(
    svg: &mut String,
    data: &ResolvedChartData,
    palette: &[String],
    _total_w: f64,
    _total_h: f64,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
) {
    // Pie icin ilk serinin degerlerini kullan (veya tum serilerin toplamlarini)
    let values: Vec<f64> = if data.series.len() == 1 {
        data.series[0].values.clone()
    } else {
        // Birden fazla seri varsa, her kategori icin toplam al
        data.categories
            .iter()
            .enumerate()
            .map(|(ci, _)| {
                data.series
                    .iter()
                    .map(|s| s.values.get(ci).copied().unwrap_or(0.0))
                    .sum()
            })
            .collect()
    };

    let total: f64 = values.iter().sum();
    if total <= 0.0 || data.categories.is_empty() {
        return;
    }

    let cx = px + pw / 2.0;
    let cy = py + ph / 2.0;
    let radius = pw.min(ph) / 2.0 * 0.65;
    let inner_frac = data.style.inner_radius.unwrap_or(0.0).clamp(0.0, 0.9);
    let inner_r = radius * inner_frac;

    let show_labels = data.labels.as_ref().is_some_and(|l| l.show);
    let label_font = data.labels.as_ref().and_then(|l| l.font_size).unwrap_or(3.0);
    let label_color = data
        .labels
        .as_ref()
        .and_then(|l| l.color.as_deref())
        .unwrap_or("#333");

    let mut start_angle = -std::f64::consts::FRAC_PI_2; // 12 o'clock

    for (i, val) in values.iter().enumerate() {
        if *val <= 0.0 {
            continue;
        }
        let sweep = (val / total) * std::f64::consts::TAU;
        let end_angle = start_angle + sweep;
        let large_arc = if sweep > std::f64::consts::PI {
            1
        } else {
            0
        };

        let x1 = cx + radius * start_angle.cos();
        let y1 = cy + radius * start_angle.sin();
        let x2 = cx + radius * end_angle.cos();
        let y2 = cy + radius * end_angle.sin();

        let color = color_at(palette,i);

        if inner_r > 0.0 {
            // Donut
            let ix1 = cx + inner_r * start_angle.cos();
            let iy1 = cy + inner_r * start_angle.sin();
            let ix2 = cx + inner_r * end_angle.cos();
            let iy2 = cy + inner_r * end_angle.sin();
            write!(
                svg,
                r##"<path d="M {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2} L {:.2} {:.2} A {:.2} {:.2} 0 {} 0 {:.2} {:.2} Z" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                x1, y1, radius, radius, large_arc, x2, y2,
                ix2, iy2, inner_r, inner_r, large_arc, ix1, iy1,
                color
            )
            .unwrap();
        } else {
            // Full pie
            write!(
                svg,
                r##"<path d="M {:.2} {:.2} L {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2} Z" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                cx, cy, x1, y1, radius, radius, large_arc, x2, y2, color
            )
            .unwrap();
        }

        // Percentage label inside slice
        if show_labels {
            let mid_angle = start_angle + sweep / 2.0;
            let label_r = if inner_r > 0.0 {
                (radius + inner_r) / 2.0
            } else {
                radius * 0.65
            };
            let lx = cx + label_r * mid_angle.cos();
            let ly = cy + label_r * mid_angle.sin();
            let pct = (val / total * 100.0).round();
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle" dominant-baseline="central">{}%</text>"##,
                lx, ly, label_font, label_color, pct
            )
            .unwrap();
        }

        // Category name label outside slice with leader line
        if i < data.categories.len() {
            let mid_angle = start_angle + sweep / 2.0;
            let line_start_r = radius; // starts at pie edge
            let line_end_r = radius + 3.0;
            let text_r = radius + 4.0;

            // Leader line from pie edge to label
            let lx1 = cx + line_start_r * mid_angle.cos();
            let ly1 = cy + line_start_r * mid_angle.sin();
            let lx2 = cx + line_end_r * mid_angle.cos();
            let ly2 = cy + line_end_r * mid_angle.sin();
            write!(
                svg,
                r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#999" stroke-width="0.2"/>"##,
                lx1, ly1, lx2, ly2
            )
            .unwrap();

            // Category text
            let tx = cx + text_r * mid_angle.cos();
            let ty = cy + text_r * mid_angle.sin();
            let anchor = if mid_angle.cos() >= 0.0 { "start" } else { "end" };
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.5" fill="#555" text-anchor="{}" dominant-baseline="central">{}</text>"##,
                tx, ty, anchor, escape_xml(&data.categories[i])
            )
            .unwrap();
        }

        start_angle = end_angle;
    }
}

fn render_legend(
    svg: &mut String,
    data: &ResolvedChartData,
    palette: &[String],
    position: &str,
    font_size: f64,
    total_w: f64,
    total_h: f64,
    margin_left: f64,
    margin_top: f64,
    plot_w: f64,
    _plot_h: f64,
) {
    let names: Vec<&str> = if matches!(data.chart_type, ChartType::Pie) {
        data.categories.iter().map(|s| s.as_str()).collect()
    } else {
        data.series.iter().map(|s| s.name.as_str()).collect()
    };

    let item_w = 3.0 + font_size * 0.4; // color rect + gap
    let spacing = 4.0;

    match position {
        "top" => {
            let y = margin_top - font_size - 1.5;
            let mut x = margin_left;
            for (i, name) in names.iter().enumerate() {
                write!(
                    svg,
                    r##"<rect x="{:.2}" y="{:.2}" width="2.5" height="2.5" fill="{}" rx="0.3"/>"##,
                    x, y - font_size * 0.3, color_at(palette,i)
                )
                .unwrap();
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="#666">{}</text>"##,
                    x + item_w, y + font_size * 0.3, font_size, escape_xml(name)
                )
                .unwrap();
                x += item_w + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
        "right" => {
            let x = margin_left + plot_w + 4.0;
            let mut y = margin_top + 2.0;
            for (i, name) in names.iter().enumerate() {
                write!(
                    svg,
                    r##"<rect x="{:.2}" y="{:.2}" width="2.5" height="2.5" fill="{}" rx="0.3"/>"##,
                    x, y, color_at(palette,i)
                )
                .unwrap();
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="#666">{}</text>"##,
                    x + item_w, y + font_size * 0.7, font_size, escape_xml(name)
                )
                .unwrap();
                y += font_size + 2.0;
            }
        }
        _ => {
            // bottom (default)
            let y = total_h - 3.0;
            let total_legend_w: f64 = names
                .iter()
                .map(|n| item_w + n.len() as f64 * font_size * 0.5 + spacing)
                .sum::<f64>()
                - spacing;
            let mut x = (total_w - total_legend_w) / 2.0;
            for (i, name) in names.iter().enumerate() {
                write!(
                    svg,
                    r##"<rect x="{:.2}" y="{:.2}" width="2.5" height="2.5" fill="{}" rx="0.3"/>"##,
                    x, y - font_size * 0.3, color_at(palette,i)
                )
                .unwrap();
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="#666">{}</text>"##,
                    x + item_w, y + font_size * 0.3, font_size, escape_xml(name)
                )
                .unwrap();
                x += item_w + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
    }
}

/// X-axis labels ortak render — bar chart icin (slot-based spacing)
fn render_x_labels(
    svg: &mut String,
    categories: &[String],
    px: f64,
    baseline_y: f64,
    pw: f64,
    n_cats: usize,
) {
    if n_cats == 0 {
        return;
    }
    let cat_width = pw / n_cats as f64;
    let max_chars = (cat_width / 1.25).max(1.0) as usize;
    let needs_rotate = categories.iter().any(|c| c.len() > max_chars);

    for (ci, cat) in categories.iter().enumerate() {
        let x = px + ci as f64 * cat_width + cat_width / 2.0;
        let y = baseline_y + 2.5;
        render_single_x_label(svg, cat, x, y, needs_rotate);
    }
}

/// X-axis labels — line chart icin (point-based spacing)
fn render_x_labels_line(
    svg: &mut String,
    categories: &[String],
    px: f64,
    baseline_y: f64,
    pw: f64,
    n_cats: usize,
) {
    if n_cats == 0 {
        return;
    }
    let spacing = if n_cats == 1 { pw } else { pw / (n_cats - 1) as f64 };
    let max_chars = (spacing / 1.25).max(1.0) as usize;
    let needs_rotate = categories.iter().any(|c| c.len() > max_chars);

    for (ci, cat) in categories.iter().enumerate() {
        let x = if n_cats == 1 {
            px + pw / 2.0
        } else {
            px + ci as f64 * pw / (n_cats - 1) as f64
        };
        let y = baseline_y + 2.5;
        render_single_x_label(svg, cat, x, y, needs_rotate);
    }
}

/// Tek bir X-axis label render — rotate gerekiyorsa -45° ile, anchor "end"
/// Anchor noktasi bar/point'in tam altinda, text sola yukari dogru uzanir
fn render_single_x_label(svg: &mut String, text: &str, x: f64, y: f64, rotate: bool) {
    if rotate {
        // -45° rotate, text-anchor="end": text, anchor noktasindan sola-yukari dogru uzanir
        // Bu sayede text asagi-sola tasmaz, sadece yukari-sola gider (plot area icinde kalir)
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="2.2" fill="#666" text-anchor="end" transform="rotate(-45,{:.2},{:.2})">{}</text>"##,
            x, y, x, y, escape_xml(text)
        )
        .unwrap();
    } else {
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="2.5" fill="#666" text-anchor="middle">{}</text>"##,
            x, y, escape_xml(text)
        )
        .unwrap();
    }
}

fn render_y_axis(
    svg: &mut String,
    min_val: f64,
    max_val: f64,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    show_grid: bool,
    grid_color: &str,
) {
    let range = if (max_val - min_val).abs() < 1e-10 {
        1.0
    } else {
        max_val - min_val
    };
    let tick_count = 5;
    for i in 0..=tick_count {
        let frac = i as f64 / tick_count as f64;
        let val = min_val + frac * range;
        let y = py + ph - frac * ph;

        // Label
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="2.3" fill="#666" text-anchor="end">{}</text>"##,
            px - 1.5,
            y + 0.8,
            format_value(val)
        )
        .unwrap();

        // Grid line
        if show_grid {
            write!(
                svg,
                r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="0.15"/>"##,
                px, y, px + pw, y, grid_color
            )
            .unwrap();
        }
    }

    // Y axis line
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        px, py, px, py + ph
    )
    .unwrap();
}

/// Tum serilerdeki min/max deger araligini bul
fn value_range(data: &ResolvedChartData, stacked: bool) -> (f64, f64) {
    if data.series.is_empty() {
        return (0.0, 1.0);
    }

    if stacked {
        let n = data.categories.len();
        let mut max_stack = 0.0_f64;
        for ci in 0..n {
            let sum: f64 = data
                .series
                .iter()
                .map(|s| s.values.get(ci).copied().unwrap_or(0.0))
                .sum();
            max_stack = max_stack.max(sum);
        }
        (0.0, max_stack * 1.05)
    } else {
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;
        for series in &data.series {
            for val in &series.values {
                min_v = min_v.min(*val);
                max_v = max_v.max(*val);
            }
        }
        // min sifirdan buyukse sifirdan basla
        if min_v > 0.0 {
            min_v = 0.0;
        }
        max_v *= 1.05;
        (min_v, max_v)
    }
}

fn format_value(v: f64) -> String {
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

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
