use crate::chart_layout::{
    self, ChartLayout, color_at, compute_bar_layout, compute_chart_layout, compute_legend,
    compute_line_layout, compute_pie_layout, format_value,
};
use crate::data_resolve::ResolvedChartData;
use std::fmt::Write;

/// mm cinsinden chart SVG uret
pub fn render_svg(data: &ResolvedChartData, width_mm: f64, height_mm: f64) -> String {
    let mut svg = String::with_capacity(4096);

    let bg = data.style.background_color.as_deref().unwrap_or("#FFFFFF");

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
    if cl.legend_show {
        render_legend(&mut svg, data, &cl, width_mm, height_mm);
    }

    // Axis labels
    let has_axis = !matches!(data.chart_type, dreport_core::models::ChartType::Pie);
    if has_axis && let Some(ref axis) = data.axis {
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
        let large_arc = if slice.sweep > std::f64::consts::PI {
            1
        } else {
            0
        };

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
        if pl.show_cat_labels && !slice.cat_label_text.is_empty() {
            write!(
                svg,
                r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#999" stroke-width="0.2"/>"##,
                slice.leader_start_x, slice.leader_start_y,
                slice.leader_end_x, slice.leader_end_y
            )
            .unwrap();

            let anchor = if slice.cat_label_anchor_end {
                "end"
            } else {
                "start"
            };
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.5" fill="#555" text-anchor="{}" dominant-baseline="central">{}</text>"##,
                slice.cat_label_x, slice.cat_label_y, anchor, escape_xml(&slice.cat_label_text)
            )
            .unwrap();
        }
    }
}

fn render_legend(
    svg: &mut String,
    data: &ResolvedChartData,
    cl: &ChartLayout,
    total_w: f64,
    total_h: f64,
) {
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
            item.text_x,
            item.text_y,
            legend.font_size,
            escape_xml(&item.name)
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
    let angle = x_labels.rotate_angle;
    for label in &x_labels.labels {
        if angle > 0.0 {
            write!(
                svg,
                r##"<text x="{:.2}" y="{:.2}" font-size="2.2" fill="#666" text-anchor="end" transform="rotate(-{:.1},{:.2},{:.2})">{}</text>"##,
                label.x, label.y, angle, label.x, label.y, escape_xml(&label.text)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_resolve::{ChartSeries, ResolvedChartData};
    use dreport_core::models::{ChartAxis, ChartLabels, ChartLegend, ChartStyle, ChartTitle, ChartType};

    fn make_bar_data(categories: Vec<&str>, series: Vec<(&str, Vec<f64>)>) -> ResolvedChartData {
        ResolvedChartData {
            chart_type: ChartType::Bar,
            categories: categories.into_iter().map(|s| s.to_string()).collect(),
            series: series
                .into_iter()
                .map(|(name, values)| ChartSeries {
                    name: name.to_string(),
                    values,
                })
                .collect(),
            title: None,
            legend: None,
            labels: None,
            axis: None,
            style: ChartStyle::default(),
            group_mode: None,
        }
    }

    fn make_line_data(categories: Vec<&str>, series: Vec<(&str, Vec<f64>)>) -> ResolvedChartData {
        let mut data = make_bar_data(categories, series);
        data.chart_type = ChartType::Line;
        data
    }

    fn make_pie_data(categories: Vec<&str>, values: Vec<f64>) -> ResolvedChartData {
        ResolvedChartData {
            chart_type: ChartType::Pie,
            categories: categories.into_iter().map(|s| s.to_string()).collect(),
            series: vec![ChartSeries {
                name: "data".to_string(),
                values,
            }],
            title: None,
            legend: None,
            labels: None,
            axis: None,
            style: ChartStyle::default(),
            group_mode: None,
        }
    }

    #[test]
    fn test_bar_chart_svg_structure() {
        let data = make_bar_data(vec!["A", "B", "C"], vec![("Sales", vec![10.0, 20.0, 30.0])]);
        let svg = render_svg(&data, 100.0, 60.0);

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        // 3 categories × 1 series = 3 bars (each with rx="0.5")
        let bar_count = svg.matches(r#"rx="0.5""#).count();
        assert_eq!(bar_count, 3, "expected 3 bars for 3 categories, got {}", bar_count);
    }

    #[test]
    fn test_bar_chart_with_labels() {
        let mut data = make_bar_data(vec!["A", "B"], vec![("S1", vec![10.0, 20.0])]);
        data.labels = Some(ChartLabels {
            show: true,
            font_size: None,
            color: None,
        });
        let svg = render_svg(&data, 100.0, 60.0);

        // Labels shown → should contain text elements with formatted values
        assert!(svg.contains("<text"), "labels enabled but no text elements found");
    }

    #[test]
    fn test_line_chart_svg_structure() {
        let data = make_line_data(vec!["Jan", "Feb", "Mar"], vec![("Revenue", vec![5.0, 15.0, 10.0])]);
        let svg = render_svg(&data, 100.0, 60.0);

        assert!(svg.starts_with("<svg"));
        // Should contain polyline for the series
        assert!(svg.contains("<polyline"), "line chart should contain polyline");
    }

    #[test]
    fn test_line_chart_with_points() {
        let mut data = make_line_data(vec!["A", "B", "C"], vec![("S1", vec![1.0, 2.0, 3.0])]);
        data.style.show_points = Some(true);
        let svg = render_svg(&data, 100.0, 60.0);

        // 3 data points → 3 circles
        let circle_count = svg.matches("<circle").count();
        assert_eq!(circle_count, 3, "expected 3 circles for 3 data points, got {}", circle_count);
    }

    #[test]
    fn test_pie_chart_svg_structure() {
        let data = make_pie_data(vec!["A", "B", "C"], vec![50.0, 30.0, 20.0]);
        let svg = render_svg(&data, 80.0, 80.0);

        assert!(svg.starts_with("<svg"));
        // 3 slices → 3 path elements
        let path_count = svg.matches("<path d=").count();
        assert_eq!(path_count, 3, "expected 3 pie slices, got {}", path_count);
    }

    #[test]
    fn test_pie_chart_percentage_labels() {
        let mut data = make_pie_data(vec!["A", "B"], vec![75.0, 25.0]);
        data.labels = Some(ChartLabels {
            show: true,
            font_size: None,
            color: None,
        });
        let svg = render_svg(&data, 80.0, 80.0);

        assert!(svg.contains("75%"), "should show 75% label");
        assert!(svg.contains("25%"), "should show 25% label");
    }

    #[test]
    fn test_legend_renders_for_multi_series() {
        let mut data = make_bar_data(
            vec!["A", "B"],
            vec![("Series 1", vec![10.0, 20.0]), ("Series 2", vec![15.0, 25.0])],
        );
        data.legend = Some(ChartLegend {
            show: true,
            position: None,
            font_size: None,
        });
        let svg = render_svg(&data, 100.0, 60.0);

        // Multi-series + legend.show → legend should render
        assert!(svg.contains("Series 1"), "legend should show series name");
        assert!(svg.contains("Series 2"), "legend should show second series name");
    }

    #[test]
    fn test_legend_hidden_for_single_series() {
        let data = make_bar_data(vec!["A", "B"], vec![("Only", vec![10.0, 20.0])]);
        let svg = render_svg(&data, 100.0, 60.0);

        // legend: None → legend_show=false → legend not rendered
        // The text "Only" might appear in x-axis labels, so check for legend swatch rect pattern
        // Legend renders swatch rects with width="2.5" height="2.5"
        let legend_swatch = svg.contains(r#"width="2.5" height="2.5""#);
        assert!(!legend_swatch, "single series should not render legend swatches");
    }

    #[test]
    fn test_empty_categories_bar_chart() {
        let data = make_bar_data(vec![], vec![("S", vec![])]);
        let svg = render_svg(&data, 100.0, 60.0);

        // Should still produce valid SVG (bg rect + no bars)
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_empty_series_bar_chart() {
        let data = make_bar_data(vec!["A", "B"], vec![]);
        let svg = render_svg(&data, 100.0, 60.0);

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_empty_pie_chart() {
        let data = make_pie_data(vec![], vec![]);
        let svg = render_svg(&data, 80.0, 80.0);

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        // No slices
        assert!(!svg.contains("<path d="), "empty pie should have no slices");
    }

    #[test]
    fn test_title_rendered() {
        let mut data = make_bar_data(vec!["A"], vec![("S", vec![10.0])]);
        data.title = Some(ChartTitle {
            text: "My Chart Title".to_string(),
            font_size: Some(4.0),
            color: Some("#333".to_string()),
            align: None,
        });
        let svg = render_svg(&data, 100.0, 60.0);

        assert!(svg.contains("My Chart Title"), "title should be rendered");
    }

    #[test]
    fn test_axis_labels_rendered() {
        let mut data = make_bar_data(vec!["Q1", "Q2"], vec![("Sales", vec![100.0, 200.0])]);
        data.axis = Some(ChartAxis {
            x_label: Some("Quarter".to_string()),
            y_label: Some("Revenue".to_string()),
            show_grid: None,
            grid_color: None,
        });
        let svg = render_svg(&data, 100.0, 60.0);

        assert!(svg.contains("Quarter"), "x axis label should be rendered");
        assert!(svg.contains("Revenue"), "y axis label should be rendered");
    }

    #[test]
    fn test_axis_labels_not_on_pie() {
        let mut data = make_pie_data(vec!["A", "B"], vec![50.0, 50.0]);
        data.axis = Some(ChartAxis {
            x_label: Some("X Label".to_string()),
            y_label: Some("Y Label".to_string()),
            show_grid: None,
            grid_color: None,
        });
        let svg = render_svg(&data, 80.0, 80.0);

        // Pie charts should not render axis labels
        assert!(!svg.contains("X Label"), "pie chart should not have x axis label");
        assert!(!svg.contains("Y Label"), "pie chart should not have y axis label");
    }

    #[test]
    fn test_escape_xml_special_chars() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<script>"), "&lt;script&gt;");
        assert_eq!(escape_xml(r#"say "hi""#), "say &quot;hi&quot;");
        assert_eq!(escape_xml("normal"), "normal");
    }

    #[test]
    fn test_donut_chart_inner_radius() {
        let mut data = make_pie_data(vec!["A", "B"], vec![60.0, 40.0]);
        data.style.inner_radius = Some(0.5);
        let svg = render_svg(&data, 80.0, 80.0);

        // Donut chart uses arc paths with inner radius → the path should contain "A" commands
        // for both outer and inner arcs
        let path_count = svg.matches("<path d=").count();
        assert_eq!(path_count, 2, "donut chart should have 2 slices");
    }
}
