//! LayoutResult → PDF bytes (krilla ile).
//! Sadece native (non-WASM) hedeflerde derlenir.

use std::collections::HashMap;

use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Size, Transform};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::{Font as KrillaFont, TextDirection};
use krilla::Document;

use crate::text_measure::TextMeasurer;
use crate::{ElementLayout, FontData, LayoutResult, PageLayout, ResolvedContent, ResolvedStyle};

/// mm → pt dönüşümü (1mm = 2.83465pt)
const MM_TO_PT: f32 = 72.0 / 25.4;

fn mm(v: f64) -> f32 {
    v as f32 * MM_TO_PT
}

/// f64 mm degerini f32 pt'ye cevir (chart render icin)
fn pt(mm_val: f64) -> f32 {
    mm_val as f32 * MM_TO_PT
}

/// Hex renk (#RRGGBB veya #RGB) → rgb::Color
fn parse_color(hex: &str) -> rgb::Color {
    let hex = hex.trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
        ),
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
            (r * 17, g * 17, b * 17)
        }
        _ => (0, 0, 0),
    };
    rgb::Color::new(r, g, b)
}

/// Rounded rectangle path oluştur. radius 0 ise düz dikdörtgen.
fn build_rect_path(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<krilla::geom::Path> {
    let mut pb = PathBuilder::new();
    if radius <= 0.0 {
        if let Some(rect) = krilla::geom::Rect::from_xywh(x, y, w, h) {
            pb.push_rect(rect);
        }
    } else {
        // Radius'u yarım kenar uzunluğuyla sınırla
        let r = radius.min(w / 2.0).min(h / 2.0);
        // Cubic bezier kappa faktörü (daire yaklaşımı)
        let k = r * 0.5522848;

        // Sağ üst köşeden başla, saat yönünde
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
        pb.line_to(x, y + r);
        pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
        pb.close();
    }
    pb.finish()
}

/// Ellipse path oluştur (4 cubic bezier ile)
fn build_ellipse_path(x: f32, y: f32, w: f32, h: f32) -> Option<krilla::geom::Path> {
    let mut pb = PathBuilder::new();
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    let kx = rx * 0.5522848;
    let ky = ry * 0.5522848;
    pb.move_to(cx, cy - ry);
    pb.cubic_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy);
    pb.cubic_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry);
    pb.cubic_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy);
    pb.cubic_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry);
    pb.close();
    pb.finish()
}

fn fill_from_color(color: rgb::Color) -> Fill {
    Fill {
        paint: color.into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    }
}

/// Font metrikleri — ascender ve descender oranları (unitsPerEm'e bölünmüş)
#[derive(Clone, Copy)]
struct FontMetrics {
    /// sTypoAscender / unitsPerEm (pozitif, genelde 0.7–1.1)
    ascender: f32,
    /// |sTypoDescender| / unitsPerEm (pozitif, genelde 0.2–0.4)
    descender: f32,
}

/// Font koleksiyonu — family + weight + italic → KrillaFont mapping
struct FontCollection {
    /// (family_lower, is_bold, is_italic) → KrillaFont
    fonts: HashMap<(String, bool, bool), KrillaFont>,
    /// Fallback font (ilk yüklenen regular)
    default: Option<KrillaFont>,
    /// Font metrikleri: (family_lower, is_bold) → FontMetrics
    metrics: HashMap<(String, bool), FontMetrics>,
}

impl FontCollection {
    fn new(font_data: &[FontData]) -> Self {
        let mut fonts = HashMap::new();
        let mut default = None;
        let mut metrics = HashMap::new();

        for fd in font_data {
            let Some(font) = KrillaFont::new(
                krilla::Data::from(fd.data.clone()),
                0,
            ) else {
                continue;
            };

            let family_lower = fd.family.to_lowercase();
            let is_bold = is_font_bold(&fd.data);
            let is_italic = is_font_italic(&fd.data);

            // Default font: ilk regular (non-bold, non-italic)
            if default.is_none() && !is_bold && !is_italic {
                default = Some(font.clone());
            }

            // Font metriklerini OS/2 tablosundan oku
            if let Some(m) = read_font_metrics(&fd.data) {
                metrics.insert((family_lower.clone(), is_bold), m);
            }

            fonts.insert((family_lower.clone(), is_bold, is_italic), font);
        }

        // Hiç regular bulamadıysak ilk font'u default yap
        if default.is_none() {
            if let Some(fd) = font_data.first() {
                default = KrillaFont::new(krilla::Data::from(fd.data.clone()), 0);
            }
        }

        Self { fonts, default, metrics }
    }

    fn get(&self, family: Option<&str>, weight: Option<&str>) -> Option<&KrillaFont> {
        let is_bold = matches!(weight, Some("bold"));
        let family_lower = family.unwrap_or("noto sans").to_lowercase();

        // Her zaman non-italic font ara (italic desteği henüz yok)
        self.fonts
            .get(&(family_lower.clone(), is_bold, false))
            .or_else(|| self.fonts.get(&(family_lower, false, false)))
            .or(self.default.as_ref())
    }

    /// CSS line-height: 1.2 modeline uygun baseline offset hesapla (pt cinsinden).
    ///
    /// CSS modeli:
    ///   content_height = (ascender + |descender|) * font_size
    ///   half_leading = (line_height - content_height) / 2
    ///   baseline_from_top = half_leading + ascender * font_size
    fn baseline_offset(&self, family: Option<&str>, weight: Option<&str>, font_size: f32) -> f32 {
        let is_bold = matches!(weight, Some("bold"));
        let family_lower = family.unwrap_or("noto sans").to_lowercase();

        let m = self.metrics
            .get(&(family_lower.clone(), is_bold))
            .or_else(|| self.metrics.get(&(family_lower, false)))
            .copied();

        match m {
            Some(m) => {
                let content_height = (m.ascender + m.descender) * font_size;
                let line_height = font_size * 1.2;
                let half_leading = (line_height - content_height) / 2.0;
                half_leading + m.ascender * font_size
            }
            None => font_size * 0.8, // Fallback
        }
    }
}

/// TTF OS/2 tablosundan font metriklerini oku
fn read_font_metrics(data: &[u8]) -> Option<FontMetrics> {
    let units_per_em = read_units_per_em(data)?;
    if units_per_em == 0 {
        return None;
    }

    let table_offset = find_os2_table(data)?;
    // sTypoAscender: offset 68 (int16), sTypoDescender: offset 70 (int16, negatif)
    if table_offset + 72 > data.len() {
        return None;
    }
    let ascender = i16::from_be_bytes([data[table_offset + 68], data[table_offset + 69]]);
    let descender = i16::from_be_bytes([data[table_offset + 70], data[table_offset + 71]]);

    Some(FontMetrics {
        ascender: ascender as f32 / units_per_em as f32,
        descender: descender.unsigned_abs() as f32 / units_per_em as f32,
    })
}

/// TTF head tablosundan unitsPerEm oku
fn read_units_per_em(data: &[u8]) -> Option<u16> {
    if data.len() < 12 {
        return None;
    }
    let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    let mut offset = 12;
    for _ in 0..num_tables {
        if offset + 16 > data.len() {
            break;
        }
        let tag = &data[offset..offset + 4];
        if tag == b"head" {
            let table_offset =
                u32::from_be_bytes([data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11]])
                    as usize;
            // unitsPerEm: head tablosunda offset 18 (uint16)
            if table_offset + 20 <= data.len() {
                return Some(u16::from_be_bytes([data[table_offset + 18], data[table_offset + 19]]));
            }
            return None;
        }
        offset += 16;
    }
    None
}

/// TTF OS/2 tablosunun offset'ini bul
fn find_os2_table(data: &[u8]) -> Option<usize> {
    if data.len() < 12 {
        return None;
    }
    let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    let mut offset = 12;
    for _ in 0..num_tables {
        if offset + 16 > data.len() {
            break;
        }
        let tag = &data[offset..offset + 4];
        if tag == b"OS/2" {
            let table_offset =
                u32::from_be_bytes([data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11]])
                    as usize;
            return Some(table_offset);
        }
        offset += 16;
    }
    None
}

/// TTF dosyasının bold olup olmadığını OS/2 tablosundan kontrol et
fn is_font_bold(data: &[u8]) -> bool {
    let Some(table_offset) = find_os2_table(data) else {
        return false;
    };
    // usWeightClass is at offset 4 in OS/2 table
    if table_offset + 6 <= data.len() {
        let weight_class = u16::from_be_bytes([data[table_offset + 4], data[table_offset + 5]]);
        return weight_class >= 700;
    }
    false
}

/// TTF dosyasının italic olup olmadığını OS/2 tablosundan kontrol et
fn is_font_italic(data: &[u8]) -> bool {
    let Some(table_offset) = find_os2_table(data) else {
        return false;
    };
    // fsSelection is at offset 62 in OS/2 table, bit 0 = ITALIC
    if table_offset + 64 <= data.len() {
        let fs_selection = u16::from_be_bytes([data[table_offset + 62], data[table_offset + 63]]);
        return fs_selection & 0x0001 != 0;
    }
    false
}

/// LayoutResult → PDF bytes
pub fn render_pdf(layout: &LayoutResult, font_data: &[FontData]) -> Result<Vec<u8>, String> {
    let fonts = FontCollection::new(font_data);
    let mut measurer = TextMeasurer::new(font_data);
    let mut doc = Document::new();

    for page in &layout.pages {
        render_page(&mut doc, page, &fonts, font_data, &mut measurer)?;
    }

    doc.finish().map_err(|e| format!("PDF oluşturma hatası: {e:?}"))
}

fn render_page(
    doc: &mut Document,
    page: &PageLayout,
    fonts: &FontCollection,
    font_data: &[FontData],
    measurer: &mut TextMeasurer,
) -> Result<(), String> {
    let w = mm(page.width_mm);
    let h = mm(page.height_mm);

    let page_settings =
        PageSettings::from_wh(w, h).ok_or_else(|| "Geçersiz sayfa boyutu".to_string())?;

    let mut pdf_page = doc.start_page_with(page_settings);
    let mut surface = pdf_page.surface();

    for el in &page.elements {
        render_element(&mut surface, el, fonts, font_data, measurer);
    }

    surface.finish();
    pdf_page.finish();
    Ok(())
}

fn render_element(
    surface: &mut krilla::surface::Surface<'_>,
    el: &ElementLayout,
    fonts: &FontCollection,
    font_data: &[FontData],
    measurer: &mut TextMeasurer,
) {
    let x = mm(el.x_mm);
    let y = mm(el.y_mm);
    let w = mm(el.width_mm);
    let h = mm(el.height_mm);

    // Container background/border
    if el.element_type == "container" {
        render_container_bg(surface, x, y, w, h, &el.style);
    }

    // Shape background/border (same visual as container bg but as leaf)
    if el.element_type == "shape" {
        render_shape(surface, x, y, w, h, &el.style, &el.content);
        return;
    }

    let Some(ref content) = el.content else {
        return;
    };

    match content {
        ResolvedContent::Text { value } => {
            render_text(surface, x, y, w, h, value, &el.style, fonts, measurer);
        }
        ResolvedContent::Line => {
            render_line(surface, x, y, w, h, &el.style);
        }
        ResolvedContent::Image { src } => {
            render_image(surface, x, y, w, h, src);
        }
        ResolvedContent::PageNumber { current, total } => {
            let text = format!("{current} / {total}");
            render_text(surface, x, y, w, h, &text, &el.style, fonts, measurer);
        }
        ResolvedContent::Table { .. } => {
            // Tablolar expand edilerek container + text olarak render edilir.
            // Bu branch'e normalde düşmemeli.
        }
        ResolvedContent::Shape { .. } => {
            // Shape zaten yukarıda render_shape() ile çizildi, buraya düşmemeli
        }
        ResolvedContent::Checkbox { checked } => {
            render_checkbox(surface, x, y, w, h, *checked, &el.style);
        }
        ResolvedContent::Barcode { format, value } => {
            render_barcode(surface, x, y, w, h, format, value, &el.style, font_data);
        }
        ResolvedContent::RichText { spans } => {
            render_rich_text(surface, x, y, w, h, spans, &el.style, fonts, measurer);
        }
        ResolvedContent::Chart { chart_data, .. } => {
            render_chart(surface, x, y, w, h, chart_data, fonts, measurer);
        }
    }
}

fn render_shape(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: &ResolvedStyle,
    content: &Option<ResolvedContent>,
) {
    let has_bg = style.background_color.is_some();
    let has_border = style.border_color.is_some() && style.border_width.unwrap_or(0.0) > 0.0;

    if !has_bg && !has_border {
        return;
    }

    let shape_type = match content {
        Some(ResolvedContent::Shape { shape_type }) => shape_type.as_str(),
        _ => "rectangle",
    };

    let rect_radius = |s: &ResolvedStyle| -> f32 {
        if shape_type == "rounded_rectangle" {
            s.border_radius.map(|r| mm(r)).unwrap_or(mm(3.0))
        } else {
            s.border_radius.map(|r| mm(r)).unwrap_or(0.0)
        }
    };

    if has_border {
        let border_width = mm(style.border_width.unwrap_or(0.5));
        let border_color = parse_color(style.border_color.as_deref().unwrap_or("#000000"));
        let inset = border_width / 2.0;

        // Fill + stroke tek path ile — anti-aliasing uyumu
        if let Some(ref bg) = style.background_color {
            surface.set_fill(Some(fill_from_color(parse_color(bg))));
        } else {
            surface.set_fill(None);
        }
        surface.set_stroke(Some(Stroke {
            paint: border_color.into(),
            width: border_width,
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));

        let path = match shape_type {
            "ellipse" => build_ellipse_path(
                x + inset, y + inset,
                w - border_width, h - border_width,
            ),
            _ => {
                let radius = rect_radius(style);
                build_rect_path(
                    x + inset, y + inset,
                    w - border_width, h - border_width,
                    (radius - inset).max(0.0),
                )
            }
        };
        if let Some(p) = path {
            surface.draw_path(&p);
        }
    } else {
        // Sadece fill, border yok
        surface.set_fill(Some(fill_from_color(parse_color(
            style.background_color.as_deref().unwrap_or("#ffffff"),
        ))));
        surface.set_stroke(None);

        let path = match shape_type {
            "ellipse" => build_ellipse_path(x, y, w, h),
            _ => build_rect_path(x, y, w, h, rect_radius(style)),
        };
        if let Some(p) = path {
            surface.draw_path(&p);
        }
    }

    surface.set_fill(None);
    surface.set_stroke(None);
}

fn render_checkbox(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    checked: bool,
    style: &ResolvedStyle,
) {
    let border_color = parse_color(style.border_color.as_deref().unwrap_or("#333333"));
    let border_width = mm(style.border_width.unwrap_or(0.3));
    let inset = border_width / 2.0;

    // Draw box outline (inset for CSS border-box match)
    surface.set_fill(None);
    surface.set_stroke(Some(Stroke {
        paint: border_color.into(),
        width: border_width,
        opacity: NormalizedF32::ONE,
        ..Default::default()
    }));

    if let Some(p) = build_rect_path(
        x + inset, y + inset,
        w - border_width, h - border_width,
        0.0,
    ) {
        surface.draw_path(&p);
    }

    // Draw checkmark if checked
    if checked {
        let check_color = parse_color(style.color.as_deref().unwrap_or("#000000"));
        let stroke_w = w.min(h) * 0.12;
        surface.set_fill(None);
        surface.set_stroke(Some(Stroke {
            paint: check_color.into(),
            width: stroke_w,
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));

        // Checkmark: two lines forming a "✓"
        let check_path = {
            let mut pb = PathBuilder::new();
            let mx = w * 0.2;
            let my = h * 0.5;
            pb.move_to(x + mx, y + my);
            pb.line_to(x + w * 0.4, y + h * 0.75);
            pb.line_to(x + w * 0.8, y + h * 0.25);
            pb.finish()
        };
        if let Some(p) = check_path {
            surface.draw_path(&p);
        }
    }

    surface.set_fill(None);
    surface.set_stroke(None);
}

fn render_container_bg(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: &ResolvedStyle,
) {
    let has_bg = style.background_color.is_some();
    let has_border = style.border_color.is_some() && style.border_width.unwrap_or(0.0) > 0.0;

    if !has_bg && !has_border {
        return;
    }

    let radius = style.border_radius.map(|r| mm(r)).unwrap_or(0.0);

    if has_border {
        let border_width = mm(style.border_width.unwrap_or(0.5));
        let border_color = parse_color(style.border_color.as_deref().unwrap_or("#000000"));
        let inset = border_width / 2.0;

        // CSS border-box: stroke path'i border_width/2 içeri çek.
        // Tek draw_path ile hem fill hem stroke çizerek anti-aliasing uyumunu sağla.
        if let Some(ref bg) = style.background_color {
            surface.set_fill(Some(fill_from_color(parse_color(bg))));
        } else {
            surface.set_fill(None);
        }
        surface.set_stroke(Some(Stroke {
            paint: border_color.into(),
            width: border_width,
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));
        if let Some(path) = build_rect_path(
            x + inset, y + inset,
            w - border_width, h - border_width,
            (radius - inset).max(0.0),
        ) {
            surface.draw_path(&path);
        }
    } else {
        // Sadece background, border yok
        surface.set_fill(Some(fill_from_color(parse_color(
            style.background_color.as_deref().unwrap_or("#ffffff"),
        ))));
        surface.set_stroke(None);
        if let Some(path) = build_rect_path(x, y, w, h, radius) {
            surface.draw_path(&path);
        }
    }

    surface.set_fill(None);
    surface.set_stroke(None);
}

fn render_text(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    text: &str,
    style: &ResolvedStyle,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    if text.is_empty() {
        return;
    }

    let font_size = style.font_size.unwrap_or(11.0) as f32;
    let color = style
        .color
        .as_deref()
        .map(parse_color)
        .unwrap_or(rgb::Color::new(0, 0, 0));

    let Some(font) = fonts.get(
        style.font_family.as_deref(),
        style.font_weight.as_deref(),
    ) else {
        return;
    };

    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);

    // Text baseline: CSS line-height 1.2 modeline uygun hesapla
    let baseline_y = y + fonts.baseline_offset(
        style.font_family.as_deref(),
        style.font_weight.as_deref(),
        font_size,
    );

    // Hizalama — cosmic-text ile text genişliğini ölçerek gerçek pozisyon hesapla
    let text_x = match style.text_align.as_deref() {
        Some("center") | Some("right") => {
            let (text_width_pt, _) = measurer.measure(
                text,
                style.font_family.as_deref(),
                font_size,
                style.font_weight.as_deref(),
                None,
            );

            if style.text_align.as_deref() == Some("center") {
                x + (w - text_width_pt) / 2.0
            } else {
                x + w - text_width_pt
            }
        }
        _ => x,
    };

    surface.draw_text(
        Point::from_xy(text_x, baseline_y),
        font.clone(),
        font_size,
        text,
        false,
        TextDirection::Auto,
    );
}

fn render_rich_text(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    spans: &[crate::ResolvedRichSpan],
    style: &ResolvedStyle,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    if spans.is_empty() {
        return;
    }

    // Varsayılan stil
    let default_font_size = style.font_size.unwrap_or(11.0) as f32;
    let default_color = style.color.as_deref().unwrap_or("#000000");
    let default_weight = style.font_weight.as_deref();
    let default_family = style.font_family.as_deref();

    // Hizalama için toplam genişliği hesapla
    let total_width = {
        let mut tw = 0.0f32;
        for span in spans {
            let fs = span.font_size.map(|f| f as f32).unwrap_or(default_font_size);
            let fw = span.font_weight.as_deref().or(default_weight);
            let ff = span.font_family.as_deref().or(default_family);
            let (sw, _) = measurer.measure(&span.text, ff, fs, fw, None);
            tw += sw;
        }
        tw
    };

    let line_start_x = match style.text_align.as_deref() {
        Some("center") => x + (w - total_width) / 2.0,
        Some("right") => x + w - total_width,
        _ => x,
    };

    // Max font size for baseline
    let max_font_size = spans
        .iter()
        .map(|s| s.font_size.map(|f| f as f32).unwrap_or(default_font_size))
        .fold(0.0f32, f32::max);
    let baseline_y = y + fonts.baseline_offset(default_family, default_weight, max_font_size);

    let mut cursor_x = line_start_x;

    for span in spans {
        if span.text.is_empty() {
            continue;
        }

        let font_size = span.font_size.map(|f| f as f32).unwrap_or(default_font_size);
        let color_str = span.color.as_deref().unwrap_or(default_color);
        let weight = span.font_weight.as_deref().or(default_weight);
        let family = span.font_family.as_deref().or(default_family);

        let color = parse_color(color_str);

        let Some(font) = fonts.get(family, weight) else {
            continue;
        };

        surface.set_fill(Some(fill_from_color(color)));
        surface.set_stroke(None);

        // Span'ın baseline'ı — farklı font boyutları için ayarla
        let span_baseline = baseline_y + (max_font_size - font_size) * 0.2;

        surface.draw_text(
            Point::from_xy(cursor_x, span_baseline),
            font.clone(),
            font_size,
            &span.text,
            false,
            TextDirection::Auto,
        );

        // Sonraki span'ın x pozisyonunu hesapla
        let (span_width, _) = measurer.measure(&span.text, family, font_size, weight, None);
        cursor_x += span_width;
    }
}

fn render_line(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: &ResolvedStyle,
) {
    let stroke_color = style
        .stroke_color
        .as_deref()
        .map(parse_color)
        .unwrap_or(rgb::Color::new(0, 0, 0));

    // Çizgiyi filled rectangle olarak çiz — CSS borderTop ile aynı davranış.
    // Stroke kullanmak sub-pixel anti-aliasing farkları yaratır.
    surface.set_fill(Some(fill_from_color(stroke_color)));
    surface.set_stroke(None);

    let rect_path = {
        let mut pb = PathBuilder::new();
        // Eleman yüksekliği layout engine tarafından stroke_width olarak hesaplandı.
        // Tüm eleman alanını dolduran ince dikdörtgen çiz.
        if let Some(rect) = krilla::geom::Rect::from_xywh(x, y, w, h) {
            pb.push_rect(rect);
        }
        pb.finish()
    };

    if let Some(p) = rect_path {
        surface.draw_path(&p);
    }

    surface.set_fill(None);
}

fn render_image(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src: &str,
) {
    if src.is_empty() {
        return;
    }

    // data:image/png;base64,... veya data:image/jpeg;base64,...
    let Some(base64_part) = src.split(',').nth(1) else {
        eprintln!("[dreport] Image src data URI değil, atlanıyor: {}...", &src[..src.len().min(60)]);
        return;
    };

    use base64::Engine;
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(base64_part) else {
        eprintln!("[dreport] Image base64 decode hatası");
        return;
    };

    // Tüm formatları image crate ile decode edip PNG'ye çevir (krilla JPEG desteği sınırlı)
    let png_data = match decode_to_png(&decoded) {
        Some(data) => data,
        None => {
            eprintln!("[dreport] Image decode/re-encode hatası, ham veri deneniyor");
            decoded
        }
    };

    embed_png(surface, x, y, w, h, &png_data);
}

fn render_barcode(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    format: &str,
    value: &str,
    style: &ResolvedStyle,
    font_data: &[FontData],
) {
    if value.is_empty() {
        return;
    }

    // Hedef piksel boyutları (yüksek çözünürlük için 4x, minimum 1px)
    let w_px = ((w * 4.0) as u32).max(1);
    let h_px = ((h * 4.0) as u32).max(1);
    let include_text = style.barcode_include_text.unwrap_or(false);

    let png_result = crate::barcode_gen::generate_barcode_png(format, value, w_px, h_px, include_text, Some(font_data));

    match png_result {
        Ok(png_bytes) => {
            embed_png(surface, x, y, w, h, &png_bytes);
        }
        Err(e) => {
            eprintln!("[dreport] Barcode üretim hatası ({format}): {e}");
        }
    }
}

/// image crate ile herhangi bir formattan decode edip PNG bytes'a çevir
fn decode_to_png(raw: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(raw).ok()?;
    let rgba = img.to_rgba8();
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        image::ExtendedColorType::Rgba8,
    )
    .ok()?;
    Some(buf)
}

/// PNG bytes'ı PDF'e göm
fn embed_png(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    data: &[u8],
) {
    let data_vec: Vec<u8> = data.to_vec();
    let Ok(img) = krilla::image::Image::from_png(data_vec.into(), true) else {
        eprintln!("[dreport] PNG krilla embed hatası");
        return;
    };

    let Some(size) = Size::from_wh(w, h) else {
        return;
    };

    surface.push_transform(&Transform::from_translate(x, y));
    surface.draw_image(img, size);
    surface.pop();
}

fn render_chart(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    data: &crate::ChartRenderData,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    // Tum hesaplar mm cinsinden yapilir, cizim pt'ye cevrilir
    let base_x_mm: f64 = (x / MM_TO_PT) as f64;
    let base_y_mm: f64 = (y / MM_TO_PT) as f64;
    let w_mm: f64 = (w / MM_TO_PT) as f64;
    let h_mm: f64 = (h / MM_TO_PT) as f64;

    // Background
    chart_rect(surface, base_x_mm, base_y_mm, w_mm, h_mm,
        parse_color(data.background_color.as_deref().unwrap_or("#FFFFFF")));

    // Margin hesaplari — SVG renderer ile AYNI mantik
    let mut margin_top = 2.0_f64;
    let mut margin_bottom = 4.0_f64;
    let mut margin_left = 8.0_f64;
    let margin_right = 4.0_f64;

    // Title
    if let Some(ref title) = data.title_text {
        if !title.is_empty() {
            let fs = data.title_font_size.unwrap_or(4.0);
            margin_top += fs * 0.4 + 2.0;
            let color = parse_color(data.title_color.as_deref().unwrap_or("#333333"));
            let font = fonts.get(None, Some("bold"));
            if let Some(f) = font {
                surface.set_fill(Some(fill_from_color(color)));
                surface.set_stroke(None);
                let fs_pt = pt(fs);
                let (tw, _) = measurer.measure(title, None, fs_pt, Some("bold"), None);
                let align = data.title_align.as_deref().unwrap_or("center");
                let tx = match align {
                    "left" => pt(base_x_mm + margin_left),
                    "right" => pt(base_x_mm + w_mm - margin_right) - tw,
                    _ => pt(base_x_mm + w_mm / 2.0) - tw / 2.0,
                };
                let ty = pt(base_y_mm + margin_top - 1.0);
                surface.draw_text(
                    Point::from_xy(tx, ty),
                    f.clone(), fs_pt, title, false, TextDirection::Auto,
                );
            }
        }
    }

    // Legend space
    let legend_show = data.legend_show;
    let legend_pos = data.legend_position.as_deref().unwrap_or("bottom");
    let legend_font = data.legend_font_size.unwrap_or(2.8);

    if legend_show && data.series.len() > 1 {
        match legend_pos {
            "top" => margin_top += legend_font + 3.0,
            "bottom" => margin_bottom += legend_font + 3.0,
            _ => {} // right — icerde handle edilecek
        }
    }

    let is_pie = matches!(data.chart_type, dreport_core::models::ChartType::Pie);

    // Axis labels icin yer ac (bar ve line) — SVG ile ayni
    if !is_pie {
        if data.x_label.is_some() {
            margin_bottom += 4.0;
        }
        if data.y_label.is_some() {
            margin_left += 4.0;
        }
        // Category labels icin alt bosluk
        let max_label_len = data.categories.iter().map(|c| c.len()).max().unwrap_or(0);
        let n_cats = data.categories.len();
        let available_w = w_mm - margin_left - margin_right;
        let cat_width = if n_cats > 0 { available_w / n_cats as f64 } else { available_w };
        let max_chars_fit = (cat_width / 1.25).max(1.0) as usize;
        let will_rotate = max_label_len > max_chars_fit;
        if will_rotate {
            let char_w_mm = 1.1;
            let max_text_w = max_label_len as f64 * char_w_mm;
            let label_v = max_text_w * 0.707;
            margin_bottom += label_v.min(25.0).max(6.0);
            let label_h = max_text_w * 0.707;
            let extra_left = (label_h - cat_width / 2.0).max(0.0);
            margin_left += extra_left.min(10.0);
        } else {
            margin_bottom += 4.0;
        }
        // Y-axis value labels icin sol bosluk
        margin_left += 6.0;
    }

    let plot_x = base_x_mm + margin_left;
    let plot_y = base_y_mm + margin_top;
    let plot_w = (w_mm - margin_left - margin_right).max(1.0);
    let plot_h = (h_mm - margin_top - margin_bottom).max(1.0);

    use dreport_core::models::ChartType;
    match data.chart_type {
        ChartType::Bar => render_chart_bar(surface, data, plot_x, plot_y, plot_w, plot_h, fonts, measurer),
        ChartType::Line => render_chart_line(surface, data, plot_x, plot_y, plot_w, plot_h, fonts, measurer),
        ChartType::Pie => render_chart_pie(surface, data, plot_x, plot_y, plot_w, plot_h, fonts, measurer),
    }

    // Legend render
    if legend_show && data.series.len() > 1 {
        render_chart_legend(surface, data, legend_pos, legend_font, base_x_mm, base_y_mm, w_mm, h_mm, margin_left, margin_top, plot_w, plot_h, fonts, measurer);
    }

    // Axis labels
    if !is_pie {
        if let Some(ref x_label) = data.x_label {
            let lx = plot_x + plot_w / 2.0;
            let ly = base_y_mm + h_mm - 2.0;
            chart_text_centered(surface, lx, ly, x_label, 2.8, "#666666", fonts, measurer);
        }
        if let Some(ref y_label) = data.y_label {
            let lx = base_x_mm + 3.0;
            let ly = plot_y + plot_h / 2.0;
            // Rotated text — krilla'da transform ile
            surface.push_transform(&Transform::from_translate(pt(lx), pt(ly)));
            surface.push_transform(&Transform::from_row(0.0, -1.0, 1.0, 0.0, 0.0, 0.0));
            chart_text_centered(surface, 0.0, 0.0, y_label, 2.8, "#666666", fonts, measurer);
            surface.pop();
            surface.pop();
        }
    }
}

/// mm degerlerini pt'ye cevirip rect ciz
fn chart_rect(surface: &mut krilla::surface::Surface<'_>, rx: f64, ry: f64, rw: f64, rh: f64, color: rgb::Color) {
    let (rx, ry, rw, rh) = (pt(rx), pt(ry), pt(rw), pt(rh));
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    let path = {
        let mut pb = PathBuilder::new();
        if let Some(r) = krilla::geom::Rect::from_xywh(rx, ry, rw, rh) {
            pb.push_rect(r);
        }
        pb.finish()
    };
    if let Some(p) = path {
        surface.draw_path(&p);
    }
}

fn chart_line_seg(surface: &mut krilla::surface::Surface<'_>, x1: f64, y1: f64, x2: f64, y2: f64, color: rgb::Color, width: f32) {
    let (x1, y1, x2, y2) = (pt(x1), pt(y1), pt(x2), pt(y2));
    surface.set_fill(None);
    surface.set_stroke(Some(Stroke {
        paint: color.into(),
        width,
        opacity: NormalizedF32::ONE,
        ..Default::default()
    }));
    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        pb.finish()
    };
    if let Some(p) = path {
        surface.draw_path(&p);
    }
}

/// Chart icin metin ciz — tek satirlik, centered
/// font_size_mm: SVG viewBox'taki mm cinsinden boyut, pt'ye cevrilir
fn chart_text_centered(
    surface: &mut krilla::surface::Surface<'_>,
    cx_mm: f64, cy_mm: f64,
    text: &str, font_size_mm: f64, color_hex: &str,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None);
    let Some(f) = font else { return; };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    let (tw, _) = measurer.measure(text, None, fs_pt, None, None);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(cx_mm) - tw / 2.0, pt(cy_mm)),
        f.clone(), fs_pt, text, false, TextDirection::Auto,
    );
}

/// Chart icin metin ciz — end-aligned (sag hizali)
fn chart_text_end(
    surface: &mut krilla::surface::Surface<'_>,
    right_x_mm: f64, cy_mm: f64,
    text: &str, font_size_mm: f64, color_hex: &str,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None);
    let Some(f) = font else { return; };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    let (tw, _) = measurer.measure(text, None, fs_pt, None, None);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(right_x_mm) - tw, pt(cy_mm)),
        f.clone(), fs_pt, text, false, TextDirection::Auto,
    );
}

/// Chart icin metin ciz — start-aligned (sol hizali)
fn chart_text_start(
    surface: &mut krilla::surface::Surface<'_>,
    x_mm: f64, cy_mm: f64,
    text: &str, font_size_mm: f64, color_hex: &str,
    fonts: &FontCollection, _measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None);
    let Some(f) = font else { return; };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(x_mm), pt(cy_mm)),
        f.clone(), fs_pt, text, false, TextDirection::Auto,
    );
}

fn chart_format_value(v: f64) -> String {
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

/// Y-axis grid + value labels (SVG render_y_axis ile ayni)
fn render_chart_y_axis(
    surface: &mut krilla::surface::Surface<'_>,
    min_val: f64, max_val: f64,
    px: f64, py: f64, pw: f64, ph: f64,
    show_grid: bool, grid_color: &str,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let range = if (max_val - min_val).abs() < 1e-10 { 1.0 } else { max_val - min_val };
    let tick_count = 5;
    for i in 0..=tick_count {
        let frac = i as f64 / tick_count as f64;
        let val = min_val + frac * range;
        let y = py + ph - frac * ph;

        // Value label
        let label = chart_format_value(val);
        chart_text_end(surface, px - 1.5, y + 0.8, &label, 2.3, "#666666", fonts, measurer);

        // Grid line
        if show_grid {
            let gc = parse_color(grid_color);
            chart_line_seg(surface, px, y, px + pw, y, gc, 0.4);
        }
    }

    // Y axis line
    let ac = parse_color("#9CA3AF");
    chart_line_seg(surface, px, py, px, py + ph, ac, 0.8);
}

/// X-axis category labels — bar chart (slot-based spacing)
fn render_chart_x_labels(
    surface: &mut krilla::surface::Surface<'_>,
    categories: &[String],
    px: f64, baseline_y: f64, pw: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let n_cats = categories.len();
    if n_cats == 0 { return; }
    let cat_width = pw / n_cats as f64;
    let max_chars = (cat_width / 1.25).max(1.0) as usize;
    let needs_rotate = categories.iter().any(|c| c.len() > max_chars);

    for (ci, cat) in categories.iter().enumerate() {
        let x = px + ci as f64 * cat_width + cat_width / 2.0;
        let y = baseline_y + 2.5;
        render_chart_single_x_label(surface, cat, x, y, needs_rotate, fonts, measurer);
    }
}

/// X-axis category labels — line chart (point-based spacing)
fn render_chart_x_labels_line(
    surface: &mut krilla::surface::Surface<'_>,
    categories: &[String],
    px: f64, baseline_y: f64, pw: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let n_cats = categories.len();
    if n_cats == 0 { return; }
    let spacing = if n_cats == 1 { pw } else { pw / (n_cats - 1) as f64 };
    let max_chars = (spacing / 1.25).max(1.0) as usize;
    let needs_rotate = categories.iter().any(|c| c.len() > max_chars);

    for (ci, cat) in categories.iter().enumerate() {
        let x = if n_cats == 1 { px + pw / 2.0 } else { px + ci as f64 * pw / (n_cats - 1) as f64 };
        let y = baseline_y + 2.5;
        render_chart_single_x_label(surface, cat, x, y, needs_rotate, fonts, measurer);
    }
}

/// Tek bir X-axis label — rotate gerekiyorsa -45° ile
fn render_chart_single_x_label(
    surface: &mut krilla::surface::Surface<'_>,
    text: &str, x_mm: f64, y_mm: f64, rotate: bool,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    if rotate {
        // -45° rotate, text-anchor="end"
        surface.push_transform(&Transform::from_translate(pt(x_mm), pt(y_mm)));
        // rotate(-45°) = cos(-45), sin(-45), -sin(-45), cos(-45)
        let c = std::f32::consts::FRAC_PI_4.cos();
        let s = std::f32::consts::FRAC_PI_4.sin();
        surface.push_transform(&Transform::from_row(c, -s, s, c, 0.0, 0.0));
        // end-aligned: text saga hizali (negatif x'e dogru)
        chart_text_end(surface, 0.0, 0.0, text, 2.2, "#666666", fonts, measurer);
        surface.pop();
        surface.pop();
    } else {
        chart_text_centered(surface, x_mm, y_mm, text, 2.5, "#666666", fonts, measurer);
    }
}

/// Legend render
fn render_chart_legend(
    surface: &mut krilla::surface::Surface<'_>,
    data: &crate::ChartRenderData,
    position: &str, font_size: f64,
    base_x: f64, base_y: f64,
    total_w: f64, total_h: f64,
    margin_left: f64, margin_top: f64,
    plot_w: f64, _plot_h: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    use dreport_core::models::ChartType;
    let names: Vec<&str> = if matches!(data.chart_type, ChartType::Pie) {
        data.categories.iter().map(|s| s.as_str()).collect()
    } else {
        data.series.iter().map(|s| s.name.as_str()).collect()
    };

    let swatch_size = 2.5;
    let item_gap = 3.0 + font_size * 0.4;
    let spacing = 4.0;

    match position {
        "top" => {
            let y = base_y + margin_top - font_size - 1.5;
            let mut x = base_x + margin_left;
            for (i, name) in names.iter().enumerate() {
                let color = parse_color(data.colors.get(i).map(|s| s.as_str()).unwrap_or("#4F46E5"));
                chart_rect(surface, x, y - font_size * 0.3, swatch_size, swatch_size, color);
                chart_text_start(surface, x + item_gap, y + font_size * 0.3, name, font_size, "#666666", fonts, measurer);
                x += item_gap + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
        "right" => {
            let x = base_x + margin_left + plot_w + 4.0;
            let mut y = base_y + margin_top + 2.0;
            for (i, name) in names.iter().enumerate() {
                let color = parse_color(data.colors.get(i).map(|s| s.as_str()).unwrap_or("#4F46E5"));
                chart_rect(surface, x, y, swatch_size, swatch_size, color);
                chart_text_start(surface, x + item_gap, y + font_size * 0.7, name, font_size, "#666666", fonts, measurer);
                y += font_size + 2.0;
            }
        }
        _ => {
            // bottom (default)
            let y = base_y + total_h - 3.0;
            let total_legend_w: f64 = names.iter()
                .map(|n| item_gap + n.len() as f64 * font_size * 0.5 + spacing)
                .sum::<f64>() - spacing;
            let mut x = base_x + (total_w - total_legend_w) / 2.0;
            for (i, name) in names.iter().enumerate() {
                let color = parse_color(data.colors.get(i).map(|s| s.as_str()).unwrap_or("#4F46E5"));
                chart_rect(surface, x, y - font_size * 0.3, swatch_size, swatch_size, color);
                chart_text_start(surface, x + item_gap, y + font_size * 0.3, name, font_size, "#666666", fonts, measurer);
                x += item_gap + name.len() as f64 * font_size * 0.5 + spacing;
            }
        }
    }
}

/// Bar chart — tum koordinatlar mm cinsinden (mutlak sayfa pozisyonu)
fn render_chart_bar(
    surface: &mut krilla::surface::Surface<'_>,
    data: &crate::ChartRenderData,
    px: f64, py: f64, pw: f64, ph: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    if data.categories.is_empty() || data.series.is_empty() { return; }

    let (min_val, max_val) = chart_value_range(data);
    let range = if (max_val - min_val).abs() < 1e-10 { 1.0 } else { max_val - min_val };

    let show_grid = data.show_grid;
    let grid_color = data.grid_color.as_deref().unwrap_or("#E5E7EB");

    // Grid + Y axis labels
    render_chart_y_axis(surface, min_val, max_val, px, py, pw, ph, show_grid, grid_color, fonts, measurer);

    let n_cats = data.categories.len();
    let n_series = data.series.len();
    let cat_width = pw / n_cats as f64;
    let bar_gap = data.bar_gap.unwrap_or(0.2).clamp(0.0, 0.8);
    let group_width = cat_width * (1.0 - bar_gap);

    let show_labels = data.show_labels;
    let label_font = data.label_font_size.unwrap_or(2.2);
    let label_color = data.label_color.as_deref().unwrap_or("#333333");

    // Bars
    if data.stacked {
        for ci in 0..n_cats {
            let mut y_off = 0.0_f64;
            for (si, series) in data.series.iter().enumerate() {
                let val = series.values.get(ci).copied().unwrap_or(0.0);
                let bh = (val / range) * ph;
                let by = py + ph - y_off - bh;
                let bx = px + ci as f64 * cat_width + cat_width * bar_gap / 2.0;
                let color = parse_color(data.colors.get(si).map(|s| s.as_str()).unwrap_or("#4F46E5"));
                chart_rect(surface, bx, by, group_width, bh.max(0.0), color);
                if show_labels && val > 0.0 {
                    let label = chart_format_value(val);
                    chart_text_centered(surface, bx + group_width / 2.0, by + bh / 2.0 + label_font * 0.15, &label, label_font, label_color, fonts, measurer);
                }
                y_off += bh;
            }
        }
    } else {
        let bar_w = group_width / n_series as f64;
        for ci in 0..n_cats {
            for (si, series) in data.series.iter().enumerate() {
                let val = series.values.get(ci).copied().unwrap_or(0.0);
                let bh = ((val - min_val) / range) * ph;
                let bx = px + ci as f64 * cat_width + cat_width * bar_gap / 2.0 + si as f64 * bar_w;
                let by = py + ph - bh;
                let color = parse_color(data.colors.get(si).map(|s| s.as_str()).unwrap_or("#4F46E5"));
                chart_rect(surface, bx, by, bar_w.max(0.1), bh.max(0.0), color);
                if show_labels {
                    let label = chart_format_value(val);
                    chart_text_centered(surface, bx + bar_w / 2.0, by - 0.8, &label, label_font, label_color, fonts, measurer);
                }
            }
        }
    }

    // X axis category labels
    render_chart_x_labels(surface, &data.categories, px, py + ph, pw, fonts, measurer);

    // X axis line
    let ac = parse_color("#9CA3AF");
    chart_line_seg(surface, px, py + ph, px + pw, py + ph, ac, 0.8);
}

/// Line chart — tum koordinatlar mm cinsinden (mutlak sayfa pozisyonu)
fn render_chart_line(
    surface: &mut krilla::surface::Surface<'_>,
    data: &crate::ChartRenderData,
    px: f64, py: f64, pw: f64, ph: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    if data.categories.is_empty() || data.series.is_empty() { return; }

    let (min_val, max_val) = chart_value_range(data);
    let range = if (max_val - min_val).abs() < 1e-10 { 1.0 } else { max_val - min_val };
    let n_cats = data.categories.len();
    let line_w = data.line_width.unwrap_or(0.5);
    let show_points = data.show_points.unwrap_or(true);

    let show_grid = data.show_grid;
    let grid_color = data.grid_color.as_deref().unwrap_or("#E5E7EB");

    // Grid + Y axis labels
    render_chart_y_axis(surface, min_val, max_val, px, py, pw, ph, show_grid, grid_color, fonts, measurer);

    let show_labels = data.show_labels;
    let label_font = data.label_font_size.unwrap_or(2.2);
    let label_color = data.label_color.as_deref().unwrap_or("#333333");

    for (si, series) in data.series.iter().enumerate() {
        let color = parse_color(data.colors.get(si).map(|s| s.as_str()).unwrap_or("#4F46E5"));

        let points: Vec<(f64, f64)> = series.values.iter().enumerate().map(|(ci, val)| {
            let xp = if n_cats == 1 { px + pw / 2.0 } else { px + ci as f64 * pw / (n_cats - 1) as f64 };
            let yp = py + ph - ((val - min_val) / range) * ph;
            (xp, yp)
        }).collect();

        // Polyline
        surface.set_fill(None);
        surface.set_stroke(Some(Stroke {
            paint: color.into(),
            width: pt(line_w),
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));
        let path = {
            let mut pb = PathBuilder::new();
            for (i, (lx, ly)) in points.iter().enumerate() {
                if i == 0 { pb.move_to(pt(*lx), pt(*ly)); }
                else { pb.line_to(pt(*lx), pt(*ly)); }
            }
            pb.finish()
        };
        if let Some(p) = path { surface.draw_path(&p); }

        // Points
        if show_points {
            for (lx, ly) in &points {
                let r = pt(0.8);
                let cx = pt(*lx);
                let cy = pt(*ly);
                surface.set_fill(Some(fill_from_color(color)));
                surface.set_stroke(None);
                let circle = {
                    let mut pb = PathBuilder::new();
                    let k = r * 0.5522848;
                    pb.move_to(cx, cy - r);
                    pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
                    pb.cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
                    pb.cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
                    pb.cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
                    pb.close();
                    pb.finish()
                };
                if let Some(p) = circle { surface.draw_path(&p); }
            }
        }

        // Value labels on points
        if show_labels {
            for (ci, val) in series.values.iter().enumerate() {
                let (lx, ly) = points[ci];
                let label = chart_format_value(*val);
                chart_text_centered(surface, lx, ly - 1.5, &label, label_font, label_color, fonts, measurer);
            }
        }
    }

    // X axis category labels
    render_chart_x_labels_line(surface, &data.categories, px, py + ph, pw, fonts, measurer);

    // Axis line
    let ac = parse_color("#9CA3AF");
    chart_line_seg(surface, px, py + ph, px + pw, py + ph, ac, 0.8);
}

/// Pie/donut chart — tum koordinatlar mm cinsinden
fn render_chart_pie(
    surface: &mut krilla::surface::Surface<'_>,
    data: &crate::ChartRenderData,
    px: f64, py: f64, pw: f64, ph: f64,
    fonts: &FontCollection, measurer: &mut TextMeasurer,
) {
    let values: Vec<f64> = if data.series.len() == 1 {
        data.series[0].values.clone()
    } else {
        data.categories.iter().enumerate().map(|(ci, _)| {
            data.series.iter().map(|s| s.values.get(ci).copied().unwrap_or(0.0)).sum()
        }).collect()
    };

    let total: f64 = values.iter().sum();
    if total <= 0.0 { return; }

    let cx = px + pw / 2.0;
    let cy = py + ph / 2.0;
    let radius = pw.min(ph) / 2.0 * 0.65;
    let inner_frac = data.inner_radius.unwrap_or(0.0).clamp(0.0, 0.9);
    let inner_r = radius * inner_frac;

    let show_labels = data.show_labels;
    let label_font = data.label_font_size.unwrap_or(3.0);
    let label_color = data.label_color.as_deref().unwrap_or("#333333");

    let mut start_angle = -std::f64::consts::FRAC_PI_2;

    for (i, val) in values.iter().enumerate() {
        if *val <= 0.0 { continue; }
        let sweep = (val / total) * std::f64::consts::TAU;
        let end_angle = start_angle + sweep;
        let color = parse_color(data.colors.get(i).map(|s| s.as_str()).unwrap_or("#4F46E5"));

        surface.set_fill(Some(fill_from_color(color)));
        surface.set_stroke(Some(Stroke {
            paint: rgb::Color::new(255, 255, 255).into(),
            width: 0.8,
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));

        let path = build_arc_path(cx, cy, radius, inner_r, start_angle, end_angle);
        if let Some(p) = path { surface.draw_path(&p); }

        // Percentage label inside slice
        if show_labels {
            let mid_angle = start_angle + sweep / 2.0;
            let label_r = if inner_r > 0.0 { (radius + inner_r) / 2.0 } else { radius * 0.65 };
            let lx = cx + label_r * mid_angle.cos();
            let ly = cy + label_r * mid_angle.sin();
            let pct = (val / total * 100.0).round();
            let label = format!("{}%", pct);
            chart_text_centered(surface, lx, ly, &label, label_font, label_color, fonts, measurer);
        }

        // Category name label outside slice with leader line
        if i < data.categories.len() {
            let mid_angle = start_angle + sweep / 2.0;
            let line_start_r = radius;
            let line_end_r = radius + 3.0;
            let text_r = radius + 4.0;

            // Leader line
            let lx1 = cx + line_start_r * mid_angle.cos();
            let ly1 = cy + line_start_r * mid_angle.sin();
            let lx2 = cx + line_end_r * mid_angle.cos();
            let ly2 = cy + line_end_r * mid_angle.sin();
            chart_line_seg(surface, lx1, ly1, lx2, ly2, parse_color("#999999"), 0.5);

            // Category text
            let tx = cx + text_r * mid_angle.cos();
            let ty = cy + text_r * mid_angle.sin();
            if mid_angle.cos() >= 0.0 {
                chart_text_start(surface, tx, ty, &data.categories[i], 2.5, "#555555", fonts, measurer);
            } else {
                chart_text_end(surface, tx, ty, &data.categories[i], 2.5, "#555555", fonts, measurer);
            }
        }

        start_angle = end_angle;
    }
}

/// Arc path olustur — pie/donut dilimi (mm cinsinden, pt'ye cevrilir)
fn build_arc_path(
    cx: f64, cy: f64,
    radius: f64, inner_r: f64,
    start: f64, end: f64,
) -> Option<krilla::geom::Path> {
    let mut pb = PathBuilder::new();

    let sx = pt(cx + radius * start.cos());
    let sy = pt(cy + radius * start.sin());

    if inner_r > 0.0 {
        pb.move_to(sx, sy);
        approximate_arc(&mut pb, cx, cy, radius, start, end);
        let ix = pt(cx + inner_r * end.cos());
        let iy = pt(cy + inner_r * end.sin());
        pb.line_to(ix, iy);
        approximate_arc(&mut pb, cx, cy, inner_r, end, start);
        pb.close();
    } else {
        pb.move_to(pt(cx), pt(cy));
        pb.line_to(sx, sy);
        approximate_arc(&mut pb, cx, cy, radius, start, end);
        pb.close();
    }

    pb.finish()
}

/// Arc'i cubic bezier segmentleriyle yaklasik ciz (her segment ≤ 90°)
fn approximate_arc(
    pb: &mut PathBuilder,
    cx: f64, cy: f64,
    r: f64,
    start: f64, end: f64,
) {
    let sweep = end - start;
    let n_segs = ((sweep.abs() / std::f64::consts::FRAC_PI_2).ceil() as usize).max(1);
    let seg_sweep = sweep / n_segs as f64;

    for i in 0..n_segs {
        let a1 = start + i as f64 * seg_sweep;
        let a2 = a1 + seg_sweep;
        let alpha = seg_sweep / 2.0;
        let cos_a = alpha.cos();
        let k = (4.0 / 3.0) * (1.0 - cos_a) / alpha.sin();

        let p2x = cx + r * a2.cos();
        let p2y = cy + r * a2.sin();
        let p1x = cx + r * a1.cos();
        let p1y = cy + r * a1.sin();

        let c1x = p1x - k * r * a1.sin();
        let c1y = p1y + k * r * a1.cos();
        let c2x = p2x + k * r * a2.sin();
        let c2y = p2y - k * r * a2.cos();

        pb.cubic_to(pt(c1x), pt(c1y), pt(c2x), pt(c2y), pt(p2x), pt(p2y));
    }
}

fn chart_value_range(data: &crate::ChartRenderData) -> (f64, f64) {
    if data.series.is_empty() {
        return (0.0, 1.0);
    }
    if data.stacked {
        let n = data.categories.len();
        let mut max_stack = 0.0_f64;
        for ci in 0..n {
            let sum: f64 = data.series.iter().map(|s| s.values.get(ci).copied().unwrap_or(0.0)).sum();
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
        if min_v > 0.0 { min_v = 0.0; }
        max_v *= 1.05;
        (min_v, max_v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElementLayout, PageLayout, ResolvedContent, ResolvedStyle};

    fn test_fonts() -> Vec<FontData> {
        crate::text_measure::load_test_fonts()
    }

    #[test]
    fn test_simple_pdf() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![
                    ElementLayout {
                        id: "title".to_string(),
                        x_mm: 15.0,
                        y_mm: 15.0,
                        width_mm: 180.0,
                        height_mm: 20.0,
                        element_type: "static_text".to_string(),
                        content: Some(ResolvedContent::Text {
                            value: "FATURA".to_string(),
                        }),
                        style: ResolvedStyle {
                            font_size: Some(18.0),
                            font_weight: Some("bold".to_string()),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    ElementLayout {
                        id: "line1".to_string(),
                        x_mm: 15.0,
                        y_mm: 38.0,
                        width_mm: 180.0,
                        height_mm: 0.5,
                        element_type: "line".to_string(),
                        content: Some(ResolvedContent::Line),
                        style: ResolvedStyle {
                            stroke_color: Some("#000000".to_string()),
                            stroke_width: Some(0.5),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                    ElementLayout {
                        id: "body".to_string(),
                        x_mm: 15.0,
                        y_mm: 42.0,
                        width_mm: 180.0,
                        height_mm: 14.0,
                        element_type: "static_text".to_string(),
                        content: Some(ResolvedContent::Text {
                            value: "Bu bir test belgesidir.".to_string(),
                        }),
                        style: ResolvedStyle {
                            font_size: Some(11.0),
                            ..Default::default()
                        },
                        children: vec![],
                    },
                ],
            }],
        };

        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("PDF oluşturulabilmeli");

        // PDF magic bytes kontrolü
        assert!(pdf.starts_with(b"%PDF"), "Geçerli PDF çıktısı olmalı");
        assert!(pdf.len() > 100, "PDF boyutu çok küçük: {} bytes", pdf.len());

        // Debug: dosyaya yaz
        let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("test_output.pdf");
        std::fs::write(&out_path, &pdf).unwrap();
        println!("Test PDF yazıldı: {}", out_path.display());
    }

    #[test]
    fn test_full_pipeline() {
        use dreport_core::models::*;
        use serde_json::json;

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            page: PageSettings { width: 210.0, height: 297.0 },
            fonts: vec!["Noto Sans".to_string()],
            header: None,
            footer: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None, min_height: None, max_width: None, max_height: None,
                },
                direction: "column".to_string(),
                gap: 5.0,
                padding: Padding { top: 15.0, right: 15.0, bottom: 15.0, left: 15.0 },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::StaticText(StaticTextElement {
                        id: "title".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None, min_height: None, max_width: None, max_height: None,
                        },
                        style: TextStyle {
                            font_size: Some(18.0),
                            font_weight: Some("bold".to_string()),
                            ..Default::default()
                        },
                        content: "FATURA".to_string(),
                    }),
                    TemplateElement::Line(LineElement {
                        id: "line1".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None, min_height: None, max_width: None, max_height: None,
                        },
                        style: LineStyle {
                            stroke_color: Some("#000000".to_string()),
                            stroke_width: Some(0.5),
                        },
                    }),
                    TemplateElement::Text(TextElement {
                        id: "firma".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None, min_height: None, max_width: None, max_height: None,
                        },
                        style: TextStyle {
                            font_size: Some(11.0),
                            ..Default::default()
                        },
                        content: None,
                        binding: dreport_core::models::ScalarBinding {
                            path: "firma.unvan".to_string(),
                        },
                    }),
                ],
            },
        };

        let data = json!({
            "firma": { "unvan": "Acme Teknoloji A.Ş." }
        });

        let fonts = test_fonts();
        let layout = crate::compute_layout(&template, &data, &fonts);
        let pdf = render_pdf(&layout, &fonts).expect("Full pipeline PDF");

        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 200);

        let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("test_output_full.pdf");
        std::fs::write(&out_path, &pdf).unwrap();
        println!("Full pipeline PDF: {}", out_path.display());
    }
}
