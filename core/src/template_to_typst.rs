use crate::models::*;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Render modu — editör önizleme vs. PDF çıktı
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Editör: layout query ekle, image placeholder göster
    Editor,
    /// PDF: layout query yok, gerçek image referansları
    Pdf,
}

// --- Ana fonksiyon ---

/// Template JSON + Data JSON → Typst markup string
pub fn template_to_typst(
    template: &Template,
    data: &serde_json::Value,
    mode: RenderMode,
) -> String {
    let mut out = String::new();
    let root = &template.root;
    let p = &root.padding;

    // Sayfa ayarları
    writeln!(
        out,
        "#set page(width: {}mm, height: {}mm, margin: (top: {}mm, right: {}mm, bottom: {}mm, left: {}mm))",
        template.page.width, template.page.height, p.top, p.right, p.bottom, p.left
    ).unwrap();
    writeln!(out).unwrap();

    // Veri enjeksiyonu
    writeln!(out, "#let data = {}", json_to_typst_dict(data)).unwrap();
    writeln!(out).unwrap();

    // Topological order ile tüm elemanlar
    let all_refs = collect_all_topological(root);

    // Barcode import — tiaoma paketi
    let barcode_formats = collect_barcode_formats(&all_refs, root);
    if !barcode_formats.is_empty() {
        let mut import_names: Vec<&str> = barcode_formats
            .iter()
            .map(|f| barcode_format_to_tiaoma(f))
            .collect();
        import_names.sort();
        import_names.dedup();
        writeln!(
            out,
            "#import \"@preview/tiaoma:0.3.0\": {}",
            import_names.join(", ")
        )
        .unwrap();
        writeln!(out).unwrap();
    }

    // Format helpers
    let mut used_formats = HashSet::new();
    for el_ref in &all_refs {
        if let ElementRef::Leaf(TemplateElement::RepeatingTable(t)) = el_ref {
            for col in &t.columns {
                if let Some(ref fmt) = col.format {
                    used_formats.insert(fmt.clone());
                }
            }
        }
    }
    let mut sorted_formats: Vec<String> = used_formats.into_iter().collect();
    sorted_formats.sort();
    for fmt in &sorted_formats {
        writeln!(out, "{}", generate_format_helper(fmt)).unwrap();
    }
    if !sorted_formats.is_empty() {
        writeln!(out).unwrap();
    }

    // Parent direction map — line elemanları için
    // Her eleman için #let tanımı
    for el_ref in &all_refs {
        let (id, content) = match el_ref {
            ElementRef::Container(c) => {
                let is_root = c.id == root.id;
                (c.id.as_str(), render_container_content(c, is_root, mode))
            }
            ElementRef::Leaf(e) => (e.id(), render_element_content(e, mode)),
        };
        writeln!(out, "#let {} = {}", id_to_var(id), content).unwrap();
    }
    // Root
    writeln!(
        out,
        "#let {} = {}",
        id_to_var(&root.id),
        render_container_content(root, true, mode)
    )
    .unwrap();
    writeln!(out).unwrap();

    // Kök container'ı renderla
    writeln!(out, "#[#{} <{}>]", id_to_var(&root.id), root.id).unwrap();

    // Layout query — sadece editör modunda
    if mode == RenderMode::Editor {
        writeln!(out).unwrap();
        write!(
            out,
            "{}",
            generate_layout_query(&all_refs, root, template.page.width)
        )
        .unwrap();
    }

    out
}

// --- Topological sort ---

enum ElementRef<'a> {
    Container(&'a ContainerElement),
    Leaf(&'a TemplateElement),
}

fn collect_all_topological(root: &ContainerElement) -> Vec<ElementRef<'_>> {
    let mut result = Vec::new();
    walk_topological(root, &mut result);
    result
}

fn walk_topological<'a>(container: &'a ContainerElement, result: &mut Vec<ElementRef<'a>>) {
    for child in &container.children {
        match child {
            TemplateElement::Container(c) => {
                walk_topological(c, result);
                result.push(ElementRef::Container(c));
            }
            other => {
                result.push(ElementRef::Leaf(other));
            }
        }
    }
}

// --- Element content rendering ---

fn render_element_content(el: &TemplateElement, mode: RenderMode) -> String {
    match el {
        TemplateElement::Container(c) => render_container_content(c, false, mode),
        TemplateElement::StaticText(e) => render_static_text_content(e),
        TemplateElement::Text(e) => render_text_content(e),
        TemplateElement::Line(e) => render_line_content(e),
        TemplateElement::RepeatingTable(e) => render_repeating_table_content(e),
        TemplateElement::Image(e) => render_image_content(e, mode),
        TemplateElement::PageNumber(e) => render_page_number_content(e),
        TemplateElement::Barcode(e) => render_barcode_content(e),
    }
}

fn render_container_content(
    el: &ContainerElement,
    skip_padding: bool,
    _mode: RenderMode,
) -> String {
    let box_params = build_box_params(el, skip_padding);

    let flow_children: Vec<&TemplateElement> = el
        .children
        .iter()
        .filter(|c| !matches!(c.position(), PositionMode::Absolute { .. }))
        .collect();
    let absolute_children: Vec<&TemplateElement> = el
        .children
        .iter()
        .filter(|c| matches!(c.position(), PositionMode::Absolute { .. }))
        .collect();

    let mut inner_parts = Vec::new();

    // Alignment hesapla
    let alignment = container_alignment_typst(&el.direction, &el.align, &el.justify);
    let is_space_between = el.justify == "space-between";

    if !flow_children.is_empty() {
        if flow_children.len() == 1 {
            let c = flow_children[0];
            inner_parts.push(format!(
                "#[#{} <{}>]",
                id_to_var(c.id()),
                c.id()
            ));
        } else if el.direction == "row" {
            // Row container → grid
            let row_align_param = match el.align.as_str() {
                "center" => Some("horizon"),
                "end" => Some("bottom"),
                "start" => Some("top"),
                _ => None, // stretch = default
            };

            if is_space_between {
                // space-between: araya 1fr spacer sütunları ekle
                let mut cols = Vec::new();
                let mut items = Vec::new();
                for (i, c) in flow_children.iter().enumerate() {
                    cols.push(size_value_to_typst(&c.size().width, true));
                    items.push(format!("    [#{} <{}>]", id_to_var(c.id()), c.id()));
                    if i < flow_children.len() - 1 {
                        cols.push("1fr".to_string());
                        items.push("    []".to_string());
                    }
                }
                let mut extra_params = String::new();
                if el.gap > 0.0 {
                    write!(extra_params, ", column-gutter: {}mm", el.gap).unwrap();
                }
                if let Some(a) = row_align_param {
                    write!(extra_params, ", align: {}", a).unwrap();
                }
                inner_parts.push(format!(
                    "#grid(columns: ({}){},\n{}\n  )",
                    cols.join(", "),
                    extra_params,
                    items.join(",\n")
                ));
            } else {
                let col_widths: Vec<String> = flow_children
                    .iter()
                    .map(|c| size_value_to_typst(&c.size().width, true))
                    .collect();
                let mut extra_params = String::new();
                if el.gap > 0.0 {
                    write!(extra_params, ", column-gutter: {}mm", el.gap).unwrap();
                }
                if let Some(a) = row_align_param {
                    write!(extra_params, ", align: {}", a).unwrap();
                }
                let items: Vec<String> = flow_children
                    .iter()
                    .map(|c| format!("    [#{} <{}>]", id_to_var(c.id()), c.id()))
                    .collect();
                inner_parts.push(format!(
                    "#grid(columns: ({}){},\n{}\n  )",
                    col_widths.join(", "),
                    extra_params,
                    items.join(",\n")
                ));
            }
        } else {
            // Column container
            if is_space_between && flow_children.len() > 1 {
                // space-between: stack yerine v(1fr) spacer'ları ile ayrı bloklar
                let mut parts = Vec::new();
                for (i, c) in flow_children.iter().enumerate() {
                    parts.push(format!("#[#{} <{}>]", id_to_var(c.id()), c.id()));
                    if i < flow_children.len() - 1 {
                        parts.push("#v(1fr)".to_string());
                    }
                }
                inner_parts.push(parts.join("\n  "));
            } else {
                let gap = if el.gap > 0.0 {
                    format!(", spacing: {}mm", el.gap)
                } else {
                    String::new()
                };
                let items: Vec<String> = flow_children
                    .iter()
                    .map(|c| format!("    [#{} <{}>]", id_to_var(c.id()), c.id()))
                    .collect();
                inner_parts.push(format!(
                    "#stack(dir: ttb{},\n{}\n  )",
                    gap,
                    items.join(",\n")
                ));
            }
        }
    }

    for child in &absolute_children {
        if let PositionMode::Absolute { x, y } = child.position() {
            inner_parts.push(format!(
                "#place(top + left, dx: {}mm, dy: {}mm)[#{} <{}>]",
                x,
                y,
                id_to_var(child.id()),
                child.id()
            ));
        }
    }

    if inner_parts.is_empty() {
        inner_parts.push("#v(5mm)".to_string());
    }

    let inner = inner_parts.join("\n  ");

    // Alignment wrapper ekle (space-between hariç, o zaten yukarıda handle edildi)
    if let Some(align_str) = alignment {
        if is_space_between {
            // space-between durumunda sadece cross-axis alignment uygula
            let cross_only = container_cross_axis_typst(&el.direction, &el.align);
            if let Some(cross) = cross_only {
                format!("box({})[\n  #align({})[{}\n  ]\n]", box_params, cross, inner)
            } else {
                format!("box({})[\n  {}\n]", box_params, inner)
            }
        } else {
            format!("box({})[\n  #align({})[{}\n  ]\n]", box_params, align_str, inner)
        }
    } else {
        format!("box({})[\n  {}\n]", box_params, inner)
    }
}

/// Container align + justify özelliklerini Typst alignment stringine dönüştür
fn container_alignment_typst(direction: &str, align: &str, justify: &str) -> Option<String> {
    let cross = container_cross_axis_typst(direction, align);

    let main = match (direction, justify) {
        (_, "start") | (_, "space-between") => None,
        ("column", "center") => Some("horizon"),
        ("column", "end") => Some("bottom"),
        ("row", "center") => Some("center"),
        ("row", "end") => Some("right"),
        _ => None,
    };

    match (cross, main) {
        (Some(c), Some(m)) => Some(format!("{} + {}", c, m)),
        (Some(c), None) => Some(c.to_string()),
        (None, Some(m)) => Some(m.to_string()),
        (None, None) => None,
    }
}

/// Sadece cross-axis alignment (row: align parametresinde kullanılır)
fn container_cross_axis_typst(direction: &str, align: &str) -> Option<&'static str> {
    match (direction, align) {
        (_, "stretch") => None,
        ("column", "start") => Some("left"),
        ("column", "center") => Some("center"),
        ("column", "end") => Some("right"),
        ("row", "start") => Some("top"),
        ("row", "center") => Some("horizon"),
        ("row", "end") => Some("bottom"),
        _ => None,
    }
}

fn render_static_text_content(el: &StaticTextElement) -> String {
    let size_params = build_box_size_params(&el.size, false);
    let text_cmd = build_text_command(&el.style, &escape_typst_content(&el.content));

    if !size_params.is_empty() {
        format!("box({})[{}]", size_params, text_cmd)
    } else {
        format!("[{}]", text_cmd)
    }
}

fn render_text_content(el: &TextElement) -> String {
    let size_params = build_box_size_params(&el.size, false);
    let data_access = format!("#data.{}", el.binding.path);
    let content = if let Some(ref prefix) = el.content {
        format!("{}{}", escape_typst_content(prefix), data_access)
    } else {
        data_access
    };
    let text_cmd = build_text_command(&el.style, &content);

    if !size_params.is_empty() {
        format!("box({})[{}]", size_params, text_cmd)
    } else {
        format!("[{}]", text_cmd)
    }
}

fn render_line_content(el: &LineElement) -> String {
    let stroke = el.style.stroke_width.unwrap_or(0.5);
    let color = el.style.stroke_color.as_deref().unwrap_or("#000000");

    match &el.size.width {
        SizeValue::Fr { .. } | SizeValue::Auto => {
            format!(
                "box(width: 1fr)[#line(length: 100%, stroke: {}pt + rgb(\"{}\"))]",
                stroke, color
            )
        }
        SizeValue::Fixed { value } => {
            format!(
                "line(length: {}mm, stroke: {}pt + rgb(\"{}\"))",
                value, stroke, color
                )
            }
        }
}

fn render_image_content(el: &ImageElement, mode: RenderMode) -> String {
    let size_params = build_box_size_params(&el.size, false);
    let placeholder_size = if size_params.is_empty() {
        "width: 40mm, height: 30mm".to_string()
    } else {
        size_params
    };

    // Dinamik binding
    if el.binding.is_some() {
        if mode == RenderMode::Pdf {
            if let Some(ref binding) = el.binding {
                return format!(
                    "box({}, fill: rgb(\"#f0f4ff\"), stroke: 0.5pt + rgb(\"#bfdbfe\"))[#align(center + horizon)[#text(size: 9pt, fill: rgb(\"#3b82f6\"))[#data.{}]]]",
                    placeholder_size, binding.path
                );
            }
        }
        return format!(
            "box({}, fill: rgb(\"#f0f4ff\"), stroke: 0.5pt + rgb(\"#bfdbfe\"))[#align(center + horizon)[#text(size: 9pt, fill: rgb(\"#3b82f6\"))[Dinamik gorsel]]]",
            placeholder_size
        );
    }

    if let Some(ref src) = el.src {
        if src.starts_with("data:") {
            if mode == RenderMode::Pdf {
                // Backend: sanal dosya referansı
                let img_filename = format!(
                    "__img_{}.dat",
                    el.id.replace(|c: char| !c.is_alphanumeric(), "_")
                );
                return format!(
                    "box({})[#image(\"{}\", width: 100%, height: 100%)]",
                    placeholder_size, img_filename
                );
            }
            // Editor: placeholder
            let mime_match = if src.contains("image/jpeg") {
                "jpg"
            } else if src.contains("image/svg+xml") {
                "svg"
            } else {
                "png"
            };
            return format!(
                "box({}, fill: rgb(\"#f8f8f8\"), stroke: 0.5pt + rgb(\"#dddddd\"))[#align(center + horizon)[#text(size: 9pt, fill: rgb(\"#888888\"))[Gorsel (.{})]]]",
                placeholder_size, mime_match
            );
        }
    }

    // Placeholder
    format!(
        "box({}, fill: rgb(\"#f0f0f0\"), stroke: 0.5pt + rgb(\"#cccccc\"))[#align(center + horizon)[#text(size: 10pt, fill: rgb(\"#999999\"))[Gorsel]]]",
        placeholder_size
    )
}

fn render_page_number_content(el: &PageNumberElement) -> String {
    let mut text_params = Vec::new();
    if let Some(fs) = el.style.font_size {
        text_params.push(format!("size: {}pt", fs));
    }
    if el.style.font_weight.as_deref() == Some("bold") {
        text_params.push("weight: \"bold\"".to_string());
    }
    if let Some(ref color) = el.style.color {
        text_params.push(format!("fill: rgb(\"{}\")", color));
    }
    let params = text_params.join(", ");

    let fmt = el.format.as_deref().unwrap_or("{current} / {total}");
    let inner = fmt
        .replace(
            "{current}",
            "\" + str(counter(page).get().first()) + \"",
        )
        .replace(
            "{total}",
            "\" + str(counter(page).final().first()) + \"",
        );
    let inner = format!("\"{}\"", inner);

    if let Some(ref align) = el.style.align {
        if align != "left" {
            return format!(
                "[#align({})[#context text({})[#({})]]]",
                align, params, inner
            );
        }
    }
    format!("[#context text({})[#({})]]", params, inner)
}

fn render_barcode_content(el: &BarcodeElement) -> String {
    let tiaoma_fn = barcode_format_to_tiaoma(&el.format);

    // Değer: statik veya binding
    let value_expr = if let Some(ref binding) = el.binding {
        format!("data.{}", binding.path)
    } else if let Some(ref value) = el.value {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        "\"\"".to_string()
    };

    // Options: renk vb.
    let mut options = Vec::new();
    if let Some(ref color) = el.style.color {
        options.push(format!("fg: rgb(\"{}\")", color));
    }
    let options_param = if options.is_empty() {
        String::new()
    } else {
        format!(", options: ({})", options.join(", "))
    };

    // Boyutu sadece width olarak geçir — tiaoma kendi aspect ratio'sunu korusun
    let w = size_value_to_typst(&el.size.width, false);
    let size_param = if w != "auto" {
        format!(", width: {}", w)
    } else {
        String::new()
    };

    format!("[#{}({}{}{})]", tiaoma_fn, value_expr, options_param, size_param)
}

/// Barcode format string → tiaoma fonksiyon adı
fn barcode_format_to_tiaoma(format: &str) -> &'static str {
    match format {
        "qr" => "qrcode",
        "ean13" | "ean8" => "ean",
        "code128" => "code128",
        "code39" => "code39",
        _ => "qrcode",
    }
}

/// Tüm barcode elemanlarının format'larını topla (import için)
fn collect_barcode_formats(all_refs: &[ElementRef<'_>], root: &ContainerElement) -> Vec<String> {
    let mut formats = HashSet::new();
    fn walk_barcode(el: &TemplateElement, formats: &mut HashSet<String>) {
        match el {
            TemplateElement::Barcode(b) => {
                formats.insert(b.format.clone());
            }
            TemplateElement::Container(c) => {
                for child in &c.children {
                    walk_barcode(child, formats);
                }
            }
            _ => {}
        }
    }
    for child in &root.children {
        walk_barcode(child, &mut formats);
    }
    // Also check non-root refs (shouldn't duplicate but be safe)
    for el_ref in all_refs {
        if let ElementRef::Leaf(TemplateElement::Barcode(b)) = el_ref {
            formats.insert(b.format.clone());
        }
    }
    let mut sorted: Vec<String> = formats.into_iter().collect();
    sorted.sort();
    sorted
}

fn render_repeating_table_content(el: &RepeatingTableElement) -> String {
    let cols = &el.columns;
    if cols.is_empty() || el.data_source.path.is_empty() {
        return "box(width: 100%)[#text(fill: rgb(\"#999999\"))[Tablo: veri kaynagi veya sutun tanimlanmamis]]".to_string();
    }

    // Sutun genislikleri
    let mut raw_col_widths: Vec<String> = cols
        .iter()
        .map(|c| match &c.width {
            SizeValue::Fixed { value } => format!("{}mm", value),
            SizeValue::Fr { value } => format!("{}fr", value),
            SizeValue::Auto => "auto".to_string(),
        })
        .collect();

    if !raw_col_widths.iter().any(|w| w.contains("fr")) {
        if let Some(pos) = raw_col_widths.iter().rposition(|w| w == "auto") {
            raw_col_widths[pos] = "1fr".to_string();
        }
    }
    let col_widths = raw_col_widths.join(", ");

    let col_aligns: Vec<&str> = cols.iter().map(|c| c.align.as_str()).collect();
    let col_aligns = col_aligns.join(", ");

    let style = &el.style;
    let header_bg = style.header_bg.as_deref().unwrap_or("#f0f0f0");
    let zebra_odd = style
        .zebra_odd
        .as_ref()
        .map(|c| format!("rgb(\"{}\")", c))
        .unwrap_or_else(|| "none".to_string());
    let zebra_even = style
        .zebra_even
        .as_ref()
        .map(|c| format!("rgb(\"{}\")", c))
        .unwrap_or_else(|| "none".to_string());
    let fill_fn = format!(
        "(_, row) => if row == 0 {{ rgb(\"{}\") }} else if calc.odd(row) {{ {} }} else {{ {} }}",
        header_bg, zebra_odd, zebra_even
    );

    let stroke_param = if let Some(ref border_color) = style.border_color {
        let bw = style.border_width.unwrap_or(0.5);
        format!(", stroke: {}pt + rgb(\"{}\")", bw, border_color)
    } else {
        String::new()
    };

    let header_color = style.header_color.as_deref().unwrap_or("#000000");
    let header_font_size = style
        .header_font_size
        .or(style.font_size)
        .unwrap_or(10.0);
    let header_cells: Vec<String> = cols
        .iter()
        .map(|c| {
            format!(
                "[#text(size: {}pt, weight: \"bold\", fill: rgb(\"{}\"))[{}]]",
                header_font_size,
                header_color,
                escape_typst_content(&c.title)
            )
        })
        .collect();
    let header_cells = header_cells.join(", ");

    let data_path = format!("data.{}", el.data_source.path);
    let font_size = style.font_size.unwrap_or(10.0);
    let row_cells: Vec<String> = cols
        .iter()
        .map(|c| {
            let accessor = format!("k.{}", c.field);
            if let Some(ref fmt) = c.format {
                format!(
                    "[#text(size: {}pt)[#{}({})]]",
                    font_size,
                    format_fn_name(fmt),
                    accessor
                )
            } else {
                format!("[#text(size: {}pt)[#{}]]", font_size, accessor)
            }
        })
        .collect();
    let row_cells = row_cells.join(", ");

    let size_params = build_box_size_params(&el.size, false);
    let box_width = if size_params.is_empty() {
        "width: 100%".to_string()
    } else {
        size_params
    };

    format!(
        "block({}, clip: false)[#block(width: 100%)[#table(\n  columns: ({}),\n  align: ({}),\n  fill: {}{},\n  {},\n  ..{}.map(k => ({})).flatten()\n)]]",
        box_width, col_widths, col_aligns, fill_fn, stroke_param, header_cells, data_path, row_cells
    )
}

// --- Layout query (editör modu) ---

fn generate_layout_query(
    all_refs: &[ElementRef<'_>],
    root: &ContainerElement,
    page_width: f64,
) -> String {
    let parent_map = build_parent_map(root);
    let width_map = compute_available_widths(root, page_width, &parent_map);

    // Row container'lardaki auto çocukların ID'lerini ve parent bilgisini topla
    // Bunlar için Typst context'inde dinamik genişlik hesabı yapacağız
    let row_child_info = collect_row_child_info(root);

    let mut var_lines = String::new();

    // Önce auto çocukları ölç (genişlik kısıtlaması olmadan)
    for info in &row_child_info {
        if info.is_auto {
            let v = id_to_var(&info.id);
            writeln!(
                var_lines,
                "  let {}auto_w = measure({}).width",
                v, v
            ).unwrap();
        }
    }

    // Row container'lar için dinamik fr genişliği hesapla
    let row_containers = collect_row_containers_with_children(root);
    for rc in &row_containers {
        let parent_v = id_to_var(&rc.container_id);
        let inner_w_var = format!("{}inner_w", parent_v);

        // Container inner width = own_width - padding
        let avail_w = width_map.get(&rc.container_id).copied().unwrap_or(page_width);
        let inner_w = avail_w - rc.padding_left - rc.padding_right;
        let inner_w_rounded = (inner_w * 100.0).round() / 100.0;

        // Auto çocukların toplam genişliğini hesapla
        let auto_sum: Vec<String> = rc.children.iter()
            .filter(|c| c.is_auto)
            .map(|c| format!("{}auto_w", id_to_var(&c.id)))
            .collect();

        let gap_mm = rc.gap * (rc.children.len().saturating_sub(1) as f64);
        let fixed_sum: f64 = rc.children.iter()
            .filter_map(|c| c.fixed_width)
            .sum();

        if auto_sum.is_empty() {
            writeln!(
                var_lines,
                "  let {} = {}mm - {}mm - {}mm",
                inner_w_var, inner_w_rounded, fixed_sum, gap_mm
            ).unwrap();
        } else {
            writeln!(
                var_lines,
                "  let {} = {}mm - {}mm - {}mm - {}",
                inner_w_var, inner_w_rounded, fixed_sum, gap_mm, auto_sum.join(" - ")
            ).unwrap();
        }

        // Her fr çocuk için gerçek genişliği hesapla
        let total_fr: f64 = rc.children.iter()
            .filter_map(|c| c.fr_value)
            .sum();
        if total_fr > 0.0 {
            for child in &rc.children {
                if let Some(fr) = child.fr_value {
                    let child_v = id_to_var(&child.id);
                    writeln!(
                        var_lines,
                        "  let {}dyn_w = {} * {} / {}",
                        child_v, inner_w_var, fr, total_fr
                    ).unwrap();
                }
            }
        }
    }

    // Tüm elemanlar için pozisyon ve boyut ölçümü
    let mut all_ids: Vec<&str> = Vec::new();
    for el_ref in all_refs {
        match el_ref {
            ElementRef::Container(c) => all_ids.push(&c.id),
            ElementRef::Leaf(e) => all_ids.push(e.id()),
        }
    }
    all_ids.push(&root.id);

    // Hangi elemanların dinamik genişliği var?
    let dyn_width_ids: HashSet<&str> = row_child_info.iter()
        .filter(|c| c.fr_value.is_some())
        .map(|c| c.id.as_str())
        .collect();

    for id in &all_ids {
        let v = id_to_var(id);
        if dyn_width_ids.contains(id) {
            // Dinamik hesaplanmış genişlik kullan
            writeln!(
                var_lines,
                "  let {}p = locate(<{}>).position()\n  let {}s = measure({}, width: {}dyn_w)\n  result += \"{}:\" + repr({}p.x) + \",\" + repr({}p.y) + \",\" + repr({}s.width) + \",\" + repr({}s.height) + \"|\"",
                v, id, v, v, v, id, v, v, v, v
            ).unwrap();
        } else {
            let avail_w = width_map.get(*id).copied().unwrap_or(page_width);
            let avail_w_rounded = (avail_w * 100.0).round() / 100.0;
            writeln!(
                var_lines,
                "  let {}p = locate(<{}>).position()\n  let {}s = measure({}, width: {}mm)\n  result += \"{}:\" + repr({}p.x) + \",\" + repr({}p.y) + \",\" + repr({}s.width) + \",\" + repr({}s.height) + \"|\"",
                v, id, v, v, avail_w_rounded, id, v, v, v, v
            ).unwrap();
        }
    }

    format!(
        "#context {{\n  let result = \"\"\n{}\n  place(bottom + right, text(size: 0.1pt, fill: white)[#result])\n}}",
        var_lines.trim_end()
    )
}

struct RowChildInfo {
    id: String,
    is_auto: bool,
    fr_value: Option<f64>,
    fixed_width: Option<f64>,
}

struct RowContainerInfo {
    container_id: String,
    padding_left: f64,
    padding_right: f64,
    gap: f64,
    children: Vec<RowChildInfo>,
}

fn collect_row_child_info(root: &ContainerElement) -> Vec<RowChildInfo> {
    let mut result = Vec::new();
    fn walk(container: &ContainerElement, result: &mut Vec<RowChildInfo>) {
        if container.direction == "row" {
            for child in &container.children {
                if matches!(child.position(), PositionMode::Absolute { .. }) {
                    continue;
                }
                let info = match &child.size().width {
                    SizeValue::Auto => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: true,
                        fr_value: None,
                        fixed_width: None,
                    },
                    SizeValue::Fr { value } => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: false,
                        fr_value: Some(*value),
                        fixed_width: None,
                    },
                    SizeValue::Fixed { value } => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: false,
                        fr_value: None,
                        fixed_width: Some(*value),
                    },
                };
                result.push(info);
            }
        }
        for child in &container.children {
            if let TemplateElement::Container(c) = child {
                walk(c, result);
            }
        }
    }
    walk(root, &mut result);
    result
}

fn collect_row_containers_with_children(root: &ContainerElement) -> Vec<RowContainerInfo> {
    let mut result = Vec::new();
    fn walk(container: &ContainerElement, result: &mut Vec<RowContainerInfo>) {
        if container.direction == "row" {
            let children: Vec<RowChildInfo> = container.children.iter()
                .filter(|c| !matches!(c.position(), PositionMode::Absolute { .. }))
                .map(|child| match &child.size().width {
                    SizeValue::Auto => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: true,
                        fr_value: None,
                        fixed_width: None,
                    },
                    SizeValue::Fr { value } => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: false,
                        fr_value: Some(*value),
                        fixed_width: None,
                    },
                    SizeValue::Fixed { value } => RowChildInfo {
                        id: child.id().to_string(),
                        is_auto: false,
                        fr_value: None,
                        fixed_width: Some(*value),
                    },
                })
                .collect();

            if children.iter().any(|c| c.fr_value.is_some()) {
                result.push(RowContainerInfo {
                    container_id: container.id.clone(),
                    padding_left: container.padding.left,
                    padding_right: container.padding.right,
                    gap: container.gap,
                    children,
                });
            }
        }
        for child in &container.children {
            if let TemplateElement::Container(c) = child {
                walk(c, result);
            }
        }
    }
    walk(root, &mut result);
    result
}

fn build_parent_map(root: &ContainerElement) -> HashMap<String, String> {
    let mut map = HashMap::new();
    fn walk(parent: &ContainerElement, map: &mut HashMap<String, String>) {
        for child in &parent.children {
            map.insert(child.id().to_string(), parent.id.clone());
            if let TemplateElement::Container(c) = child {
                walk(c, map);
            }
        }
    }
    walk(root, &mut map);
    map
}

fn compute_available_widths(
    root: &ContainerElement,
    page_width: f64,
    _parent_map: &HashMap<String, String>,
) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    let root_content_width = page_width - root.padding.left - root.padding.right;
    map.insert(root.id.clone(), root_content_width);

    fn get_container_inner_width(
        c: &ContainerElement,
        root_id: &str,
        root_content_width: f64,
        map: &HashMap<String, f64>,
    ) -> f64 {
        let own_width = map.get(&c.id).copied().unwrap_or(root_content_width);
        if c.id == root_id {
            return own_width;
        }
        own_width - c.padding.left - c.padding.right
    }

    fn walk(
        container: &ContainerElement,
        root_id: &str,
        root_content_width: f64,
        map: &mut HashMap<String, f64>,
    ) {
        let inner_w =
            get_container_inner_width(container, root_id, root_content_width, map);

        if container.direction == "column" {
            for child in &container.children {
                let child_w = match &child.size().width {
                    SizeValue::Fixed { value } => *value,
                    _ => inner_w,
                };
                map.insert(child.id().to_string(), child_w);
                if let TemplateElement::Container(c) = child {
                    walk(c, root_id, root_content_width, map);
                }
            }
        } else {
            // row
            let mut used_width = 0.0;
            let mut total_fr = 0.0;
            let gap = container.gap
                * (container.children.len().saturating_sub(1) as f64);

            for child in &container.children {
                match &child.size().width {
                    SizeValue::Fixed { value } => used_width += value,
                    SizeValue::Fr { value } => total_fr += value,
                    _ => {}
                }
            }

            let remaining_w = (inner_w - used_width - gap).max(0.0);

            for child in &container.children {
                let child_w = match &child.size().width {
                    SizeValue::Fixed { value } => *value,
                    SizeValue::Fr { value } => {
                        if total_fr > 0.0 {
                            (value / total_fr) * remaining_w
                        } else {
                            remaining_w
                        }
                    }
                    SizeValue::Auto => inner_w,
                };
                map.insert(child.id().to_string(), child_w);
                if let TemplateElement::Container(c) = child {
                    walk(c, root_id, root_content_width, map);
                }
            }
        }
    }

    walk(root, &root.id, root_content_width, &mut map);
    map
}

// --- Image dosya cikarimi ---

/// Template'deki base64 image'lari cikar
pub fn extract_image_files(template: &Template) -> HashMap<String, Vec<u8>> {
    let mut files = HashMap::new();
    collect_images(
        &TemplateElement::Container(template.root.clone()),
        &mut files,
    );
    files
}

fn collect_images(el: &TemplateElement, files: &mut HashMap<String, Vec<u8>>) {
    match el {
        TemplateElement::Image(img) => {
            if let Some(ref src) = img.src {
                if src.starts_with("data:") {
                    if let Some(b64_data) = src.split(',').nth(1) {
                        if let Ok(bytes) = BASE64.decode(b64_data) {
                            let filename = format!(
                                "__img_{}.dat",
                                img.id
                                    .replace(|c: char| !c.is_alphanumeric(), "_")
                            );
                            files.insert(filename, bytes);
                        }
                    }
                }
            }
        }
        TemplateElement::Container(c) => {
            for child in &c.children {
                collect_images(child, files);
            }
        }
        _ => {}
    }
}

// --- Format helpers ---

fn format_fn_name(fmt: &str) -> &str {
    match fmt {
        "currency" => "format-currency",
        "date" => "format-date",
        "percentage" => "format-pct",
        "number" => "format-num",
        _ => "str",
    }
}

fn generate_format_helper(fmt: &str) -> String {
    match fmt {
        "currency" => r#"#let format-currency(v) = {
  let s = str(calc.round(float(v) * 100) / 100)
  let parts = s.split(".")
  let int-part = parts.at(0)
  let dec-part = if parts.len() > 1 { parts.at(1) } else { "00" }
  if dec-part.len() < 2 { dec-part = dec-part + "0" }
  let digits = int-part.clusters()
  let grouped = ""
  let count = 0
  for d in digits.rev() {
    if count > 0 and calc.rem(count, 3) == 0 { grouped = "." + grouped }
    grouped = d + grouped
    count = count + 1
  }
  grouped + "," + dec-part + " \u{20BA}"
}"#
        .to_string(),
        "date" => r#"#let format-date(v) = {
  let parts = str(v).split("-")
  if parts.len() == 3 { parts.at(2) + "." + parts.at(1) + "." + parts.at(0) } else { str(v) }
}"#
        .to_string(),
        "percentage" => {
            r#"#let format-pct(v) = { str(calc.round(float(v) * 100) / 100) + "%" }"#
                .to_string()
        }
        "number" => r#"#let format-num(v) = {
  let s = str(v)
  let parts = s.split(".")
  let int-part = parts.at(0)
  let digits = int-part.clusters()
  let grouped = ""
  let count = 0
  for d in digits.rev() {
    if count > 0 and calc.rem(count, 3) == 0 { grouped = "." + grouped }
    grouped = d + grouped
    count = count + 1
  }
  if parts.len() > 1 { grouped + "," + parts.at(1) } else { grouped }
}"#
        .to_string(),
        _ => String::new(),
    }
}

// --- Yardimcilar ---

fn id_to_var(id: &str) -> String {
    let mut result = String::from("v_");
    for c in id.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    result
}

fn build_box_params(el: &ContainerElement, skip_padding: bool) -> String {
    let mut parts = Vec::new();

    let size_params = build_box_size_params(&el.size, false);
    if !size_params.is_empty() {
        parts.push(size_params);
    }

    if !skip_padding {
        let p = &el.padding;
        let has_padding = p.top > 0.0 || p.right > 0.0 || p.bottom > 0.0 || p.left > 0.0;
        if has_padding {
            parts.push(format!(
                "inset: (top: {}mm, right: {}mm, bottom: {}mm, left: {}mm)",
                p.top, p.right, p.bottom, p.left
            ));
        }
    }

    let style_params = build_container_style_params(el);
    if !style_params.is_empty() {
        parts.push(style_params);
    }

    parts.join(", ")
}

fn build_box_size_params(size: &SizeConstraint, allow_fr: bool) -> String {
    let mut parts = Vec::new();
    let w = size_value_to_typst(&size.width, allow_fr);
    if w != "auto" {
        parts.push(format!("width: {}", w));
    }
    let h = size_value_to_typst(&size.height, allow_fr);
    if h != "auto" {
        parts.push(format!("height: {}", h));
    }
    parts.join(", ")
}

fn size_value_to_typst(sv: &SizeValue, allow_fr: bool) -> String {
    match sv {
        SizeValue::Fixed { value } => format!("{}mm", value),
        SizeValue::Auto => "auto".to_string(),
        SizeValue::Fr { value } => {
            if allow_fr {
                format!("{}fr", value)
            } else {
                "100%".to_string()
            }
        }
    }
}

fn build_container_style_params(el: &ContainerElement) -> String {
    let mut parts = Vec::new();
    if let Some(ref bg) = el.style.background_color {
        parts.push(format!("fill: rgb(\"{}\")", bg));
    }
    let bw = el.style.border_width.unwrap_or(0.0);
    let has_border = bw > 0.0 || el.style.border_color.is_some();
    if has_border {
        let width = if bw > 0.0 { bw } else { 1.0 };
        let color = el.style.border_color.as_deref().unwrap_or("#000000");
        let dash_part = match el.style.border_style.as_deref() {
            Some("dashed") => ", dash: \"dashed\"",
            Some("dotted") => ", dash: \"dotted\"",
            _ => "",
        };
        parts.push(format!(
            "stroke: (paint: rgb(\"{}\"), thickness: {}pt{})",
            color, width, dash_part
        ));
    }
    if let Some(br) = el.style.border_radius {
        if br > 0.0 {
            parts.push(format!("radius: {}pt", br));
        }
    }
    parts.join(", ")
}

fn build_text_command(style: &TextStyle, content: &str) -> String {
    let mut parts = Vec::new();
    if let Some(fs) = style.font_size {
        parts.push(format!("size: {}pt", fs));
    }
    if style.font_weight.as_deref() == Some("bold") {
        parts.push("weight: \"bold\"".to_string());
    }
    if let Some(ref ff) = style.font_family {
        parts.push(format!("font: \"{}\"", ff));
    }
    if let Some(ref color) = style.color {
        parts.push(format!("fill: rgb(\"{}\")", color));
    }

    let params = parts.join(", ");
    let mut result = format!("#text({})[{}]", params, content);

    if let Some(ref align) = style.align {
        if align != "left" {
            result = format!("#align({})[{}]", align, result);
        }
    }
    result
}

fn escape_typst_content(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('@', "\\@")
        .replace('<', "\\<")
        .replace('>', "\\>")
}

fn json_to_typst_dict(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "none".to_string(),
        serde_json::Value::Bool(b) => {
            if *b { "true" } else { "false" }.to_string()
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            format!("\"{}\"", s.replace('"', "\\\""))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_typst_dict).collect();
            format!("({},)", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, json_to_typst_dict(v)))
                .collect();
            if entries.is_empty() {
                "(:)".to_string()
            } else {
                format!("({})", entries.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_row_container_fr_children() {
        let template = Template {
            id: "test".into(),
            name: "Test".into(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec![],
            root: ContainerElement {
                id: "root".into(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None, min_height: None, max_width: None, max_height: None,
                },
                direction: "column".into(),
                gap: 5.0,
                padding: Padding { top: 15.0, right: 15.0, bottom: 15.0, left: 15.0 },
                align: "stretch".into(),
                justify: "start".into(),
                style: ContainerStyle::default(),
                children: vec![
                    TemplateElement::Container(ContainerElement {
                        id: "row1".into(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None, min_height: None, max_width: None, max_height: None,
                        },
                        direction: "row".into(),
                        gap: 5.0,
                        padding: Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 },
                        align: "start".into(),
                        justify: "start".into(),
                        style: ContainerStyle::default(),
                        children: vec![
                            TemplateElement::Container(ContainerElement {
                                id: "child1".into(),
                                position: PositionMode::Flow,
                                size: SizeConstraint {
                                    width: SizeValue::Fr { value: 1.0 },
                                    height: SizeValue::Auto,
                                    min_width: None, min_height: None, max_width: None, max_height: None,
                                },
                                direction: "column".into(),
                                gap: 0.0,
                                padding: Padding { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 },
                                align: "start".into(),
                                justify: "start".into(),
                                style: ContainerStyle::default(),
                                children: vec![
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "txt1".into(),
                                        position: PositionMode::Flow,
                                        size: SizeConstraint {
                                            width: SizeValue::Auto,
                                            height: SizeValue::Auto,
                                            min_width: None, min_height: None, max_width: None, max_height: None,
                                        },
                                        style: TextStyle { font_size: Some(11.0), ..Default::default() },
                                        content: "Sol".into(),
                                    }),
                                ],
                            }),
                            TemplateElement::Container(ContainerElement {
                                id: "child2".into(),
                                position: PositionMode::Flow,
                                size: SizeConstraint {
                                    width: SizeValue::Fr { value: 1.0 },
                                    height: SizeValue::Auto,
                                    min_width: None, min_height: None, max_width: None, max_height: None,
                                },
                                direction: "column".into(),
                                gap: 0.0,
                                padding: Padding { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 },
                                align: "start".into(),
                                justify: "start".into(),
                                style: ContainerStyle::default(),
                                children: vec![
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "txt2".into(),
                                        position: PositionMode::Flow,
                                        size: SizeConstraint {
                                            width: SizeValue::Auto,
                                            height: SizeValue::Auto,
                                            min_width: None, min_height: None, max_width: None, max_height: None,
                                        },
                                        style: TextStyle { font_size: Some(11.0), ..Default::default() },
                                        content: "Sag".into(),
                                    }),
                                ],
                            }),
                        ],
                    }),
                ],
            },
        };

        let data = serde_json::json!({});
        let result = template_to_typst(&template, &data, RenderMode::Editor);
        println!("=== GENERATED TYPST ===\n{}", result);

        // Grid sütunları 1fr kullanmalı, box'lar 100% kullanmalı
        assert!(result.contains("columns: (1fr, 1fr)"), "Grid columns should use 1fr:\n{}", result);
        assert!(result.contains("box(width: 100%"), "Boxes should use 100%:\n{}", result);
    }
}
