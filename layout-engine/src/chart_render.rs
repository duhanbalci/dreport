use crate::chart_layout::{
    self, color_at, compute_bar_layout, compute_chart_layout, compute_legend,
    compute_line_layout, compute_pie_layout, format_value, ChartLayout,
};
use crate::data_resolve::ResolvedChartData;
use std::fmt::Write;

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

    let cl = compute_chart_layout(data, width_mm, height_mm, 0.0, 0.0);

    // Title
    if let Some(ref title) = cl.title {
        let anchor = match title.align.as_str() {
            "left" => "start",
            "right" => "end",
            _ => "middle",
        };
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="{}" font-weight="bold">{}</text>"##,
            title.x, title.y, title.font_size, title.color, anchor, escape_xml(&title.text)
        )
        .unwrap();
    }

    match data.chart_type {
        dreport_core::models::ChartType::Bar => render_bar(&mut svg, data, &cl),
        dreport_core::models::ChartType::Line => render_line(&mut svg, data, &cl),
        dreport_core::models::ChartType::Pie => render_pie(&mut svg, data, &cl),
    }

    // Legend render
    if cl.legend_show && data.series.len() > 1 {
        render_legend(&mut svg, data, &cl, width_mm, height_mm);
    }

    // Axis labels
    let has_axis = !matches!(data.chart_type, dreport_core::models::ChartType::Pie);
    if has_axis {
        if let Some(ref axis) = data.axis {
            if let Some(ref x_label) = axis.x_label {
                let x = cl.plot_x + cl.plot_w / 2.0;
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
                let y = cl.plot_y + cl.plot_h / 2.0;
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

fn render_bar(svg: &mut String, data: &ResolvedChartData, cl: &ChartLayout) {
    if data.categories.is_empty() || data.series.is_empty() {
        return;
    }

    let bl = compute_bar_layout(data, cl);

    // Y axis
    render_y_axis_svg(svg, &bl.y_axis);

    // Bars
    for bar in &bl.bars {
        let color = color_at(&cl.palette, bar.color_idx);
        write!(
            svg,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" rx="0.5"/>"##,
            bar.x, bar.y, bar.w, bar.h, color
        )
        .unwrap();

        if bl.show_labels {
            if bl.stacked {
                if bar.value > 0.0 {
                    write!(
                        svg,
                        r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                        bar.label_x, bar.label_y, bl.label_font, bl.label_color, format_value(bar.value)
                    )
                    .unwrap();
                }
            } else {
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                    bar.label_x, bar.label_y, bl.label_font, bl.label_color, format_value(bar.value)
                )
                .unwrap();
            }
        }
    }

    // X axis labels
    render_x_labels_svg(svg, &bl.x_labels);

    // X axis line
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        bl.x_axis_x1, bl.x_axis_y, bl.x_axis_x2, bl.x_axis_y
    )
    .unwrap();
}

fn render_line(svg: &mut String, data: &ResolvedChartData, cl: &ChartLayout) {
    if data.categories.is_empty() || data.series.is_empty() {
        return;
    }

    let ll = compute_line_layout(data, cl);

    // Y axis
    render_y_axis_svg(svg, &ll.y_axis);

    for series_layout in &ll.series {
        let color = color_at(&cl.palette, series_layout.color_idx);
        let mut points = String::new();
        let mut point_circles = String::new();

        for pt in &series_layout.points {
            write!(points, "{:.2},{:.2} ", pt.x, pt.y).unwrap();

            if ll.show_points {
                write!(
                    point_circles,
                    r##"<circle cx="{:.2}" cy="{:.2}" r="0.8" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                    pt.x, pt.y, color
                )
                .unwrap();
            }

            if ll.show_labels {
                write!(
                    svg,
                    r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle">{}</text>"##,
                    pt.x, pt.y - 1.5, ll.label_font, ll.label_color, format_value(pt.value)
                )
                .unwrap();
            }
        }

        write!(
            svg,
            r##"<polyline points="{}" fill="none" stroke="{}" stroke-width="{:.2}" stroke-linejoin="round" stroke-linecap="round"/>"##,
            points.trim(), color, ll.line_width
        )
        .unwrap();
        svg.push_str(&point_circles);
    }

    // X axis labels
    render_x_labels_svg(svg, &ll.x_labels);

    // Axis line
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        ll.x_axis_x1, ll.x_axis_y, ll.x_axis_x2, ll.x_axis_y
    )
    .unwrap();
}

fn render_pie(svg: &mut String, data: &ResolvedChartData, cl: &ChartLayout) {
    let pl = compute_pie_layout(data, cl);

    if pl.slices.is_empty() {
        return;
    }

    let cx = pl.cx;
    let cy = pl.cy;
    let radius = pl.radius;
    let inner_r = pl.inner_radius;

    for slice in &pl.slices {
        let color = color_at(&cl.palette, slice.color_idx);
        let large_arc = if slice.sweep > std::f64::consts::PI { 1 } else { 0 };

        let x1 = cx + radius * slice.start_angle.cos();
        let y1 = cy + radius * slice.start_angle.sin();
        let x2 = cx + radius * slice.end_angle.cos();
        let y2 = cy + radius * slice.end_angle.sin();

        if inner_r > 0.0 {
            let ix1 = cx + inner_r * slice.start_angle.cos();
            let iy1 = cy + inner_r * slice.start_angle.sin();
            let ix2 = cx + inner_r * slice.end_angle.cos();
            let iy2 = cy + inner_r * slice.end_angle.sin();
            write!(
                svg,
                r##"<path d="M {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2} L {:.2} {:.2} A {:.2} {:.2} 0 {} 0 {:.2} {:.2} Z" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                x1, y1, radius, radius, large_arc, x2, y2,
                ix2, iy2, inner_r, inner_r, large_arc, ix1, iy1,
                color
            )
            .unwrap();
        } else {
            write!(
                svg,
                r##"<path d="M {:.2} {:.2} L {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2} Z" fill="{}" stroke="white" stroke-width="0.3"/>"##,
                cx, cy, x1, y1, radius, radius, large_arc, x2, y2, color
            )
            .unwrap();
        }

        // Percentage label inside slice
        if pl.show_labels {
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="{}" text-anchor="middle" dominant-baseline="central">{}%</text>"##,
                slice.label_x, slice.label_y, pl.label_font, pl.label_color,
                (slice.fraction * 100.0).round()
            )
            .unwrap();
        }

        // Category name label outside slice with leader line
        if !slice.cat_label_text.is_empty() {
            write!(
                svg,
                r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#999" stroke-width="0.2"/>"##,
                slice.leader_start_x, slice.leader_start_y,
                slice.leader_end_x, slice.leader_end_y
            )
            .unwrap();

            let anchor = if slice.cat_label_anchor_end { "end" } else { "start" };
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.5" fill="#555" text-anchor="{}" dominant-baseline="central">{}</text>"##,
                slice.cat_label_x, slice.cat_label_y, anchor, escape_xml(&slice.cat_label_text)
            )
            .unwrap();
        }
    }
}

fn render_legend(svg: &mut String, data: &ResolvedChartData, cl: &ChartLayout, total_w: f64, total_h: f64) {
    let legend = compute_legend(data, cl, 0.0, 0.0, total_w, total_h);

    for item in &legend.items {
        let color = color_at(&cl.palette, item.color_idx);
        write!(
            svg,
            r##"<rect x="{:.2}" y="{:.2}" width="2.5" height="2.5" fill="{}" rx="0.3"/>"##,
            item.swatch_x, item.swatch_y, color
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="{:.1}" fill="#666">{}</text>"##,
            item.text_x, item.text_y, legend.font_size, escape_xml(&item.name)
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// SVG-specific helper renderers that consume shared layout structs
// ---------------------------------------------------------------------------

fn render_y_axis_svg(svg: &mut String, y_axis: &chart_layout::YAxisLayout) {
    for tick in &y_axis.ticks {
        write!(
            svg,
            r##"<text x="{:.2}" y="{:.2}" font-size="2.3" fill="#666" text-anchor="end">{}</text>"##,
            y_axis.axis_x - 1.5, tick.y + 0.8, tick.label
        )
        .unwrap();

        if y_axis.show_grid {
            write!(
                svg,
                r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="0.15"/>"##,
                y_axis.axis_x, tick.y, y_axis.grid_end_x, tick.y, y_axis.grid_color
            )
            .unwrap();
        }
    }

    // Y axis line
    write!(
        svg,
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#9CA3AF" stroke-width="0.3"/>"##,
        y_axis.axis_x, y_axis.axis_y_start, y_axis.axis_x, y_axis.axis_y_end
    )
    .unwrap();
}

fn render_x_labels_svg(svg: &mut String, x_labels: &chart_layout::XLabelLayout) {
    for label in &x_labels.labels {
        if x_labels.needs_rotate {
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.2" fill="#666" text-anchor="end" transform="rotate(-45,{:.2},{:.2})">{}</text>"##,
                label.x, label.y, label.x, label.y, escape_xml(&label.text)
            )
            .unwrap();
        } else {
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.5" fill="#666" text-anchor="middle">{}</text>"##,
                label.x, label.y, escape_xml(&label.text)
            )
            .unwrap();
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
