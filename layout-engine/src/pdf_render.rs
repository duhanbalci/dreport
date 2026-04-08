//! LayoutResult → PDF bytes (krilla ile).
//! Sadece native (non-WASM) hedeflerde derlenir.

use std::collections::HashMap;

use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Size, Transform};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::{Font as KrillaFont, TextDirection};

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
            let Some(font) = KrillaFont::new(krilla::Data::from(fd.data.clone()), 0) else {
                continue;
            };

            let family_lower = fd.family.to_lowercase();
            let is_bold = fd.is_bold();
            let is_italic = fd.italic;

            // Default font: ilk regular (non-bold, non-italic)
            if default.is_none() && !is_bold && !is_italic {
                default = Some(font.clone());
            }

            // Font metriklerini font_meta'dan oku
            if let Some(meta) = crate::font_meta::parse_font_meta(&fd.data) {
                let units_per_em = meta.units_per_em;
                if units_per_em > 0 {
                    metrics.insert(
                        (family_lower.clone(), is_bold),
                        FontMetrics {
                            ascender: meta.ascender as f32 / units_per_em as f32,
                            descender: meta.descender.unsigned_abs() as f32 / units_per_em as f32,
                        },
                    );
                }
            }

            fonts.insert((family_lower.clone(), is_bold, is_italic), font);
        }

        // Hiç regular bulamadıysak ilk font'u default yap
        if default.is_none()
            && let Some(fd) = font_data.first()
        {
            default = KrillaFont::new(krilla::Data::from(fd.data.clone()), 0);
        }

        Self {
            fonts,
            default,
            metrics,
        }
    }

    fn get(
        &self,
        family: Option<&str>,
        weight: Option<&str>,
        font_style: Option<&str>,
    ) -> Option<&KrillaFont> {
        let is_bold = matches!(weight, Some("bold"));
        let is_italic = matches!(font_style, Some("italic"));
        let family_lower = family.unwrap_or("noto sans").to_lowercase();

        self.fonts
            .get(&(family_lower.clone(), is_bold, is_italic))
            .or_else(|| self.fonts.get(&(family_lower.clone(), is_bold, false)))
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

        let m = self
            .metrics
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

// OS/2 table parsing is now handled by font_meta module.
// FontData.weight and FontData.italic carry pre-parsed metadata.

/// LayoutResult → PDF bytes
pub fn render_pdf(layout: &LayoutResult, font_data: &[FontData]) -> Result<Vec<u8>, String> {
    let fonts = FontCollection::new(font_data);
    let mut measurer = TextMeasurer::new(font_data);
    let mut doc = Document::new();

    for page in &layout.pages {
        render_page(&mut doc, page, &fonts, font_data, &mut measurer)?;
    }

    doc.finish()
        .map_err(|e| format!("PDF oluşturma hatası: {e:?}"))
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
            s.border_radius.map(mm).unwrap_or(mm(3.0))
        } else {
            s.border_radius.map(mm).unwrap_or(0.0)
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
            "ellipse" => {
                build_ellipse_path(x + inset, y + inset, w - border_width, h - border_width)
            }
            _ => {
                let radius = rect_radius(style);
                build_rect_path(
                    x + inset,
                    y + inset,
                    w - border_width,
                    h - border_width,
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
        x + inset,
        y + inset,
        w - border_width,
        h - border_width,
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

    let radius = style.border_radius.map(mm).unwrap_or(0.0);

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
            x + inset,
            y + inset,
            w - border_width,
            h - border_width,
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

#[allow(clippy::too_many_arguments)]
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
        style.font_style.as_deref(),
    ) else {
        return;
    };

    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);

    // Text baseline: CSS line-height 1.2 modeline uygun hesapla
    let baseline_offset = fonts.baseline_offset(
        style.font_family.as_deref(),
        style.font_weight.as_deref(),
        font_size,
    );

    // cosmic-text ile text'i satırlara böl (wrapping)
    let lines = measurer.layout_lines(
        text,
        style.font_family.as_deref(),
        font_size,
        style.font_weight.as_deref(),
        w,
    );

    if lines.is_empty() {
        return;
    }

    for line in &lines {
        if line.text.is_empty() {
            continue;
        }

        let line_y = y + line.y_offset_pt + baseline_offset;

        // Hizalama
        let line_x = match style.text_align.as_deref() {
            Some("center") => x + (w - line.width_pt) / 2.0,
            Some("right") => x + w - line.width_pt,
            _ => x,
        };

        surface.draw_text(
            Point::from_xy(line_x, line_y),
            font.clone(),
            font_size,
            &line.text,
            false,
            TextDirection::Auto,
        );
    }
}

#[allow(clippy::too_many_arguments)]
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
            let fs = span
                .font_size
                .map(|f| f as f32)
                .unwrap_or(default_font_size);
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

        let font_size = span
            .font_size
            .map(|f| f as f32)
            .unwrap_or(default_font_size);
        let color_str = span.color.as_deref().unwrap_or(default_color);
        let weight = span.font_weight.as_deref().or(default_weight);
        let family = span.font_family.as_deref().or(default_family);

        let color = parse_color(color_str);

        let Some(font) = fonts.get(family, weight, None) else {
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

#[derive(Debug, PartialEq)]
enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Unknown,
}

fn detect_image_format(data: &[u8]) -> ImageFormat {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        ImageFormat::Png
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        ImageFormat::Jpeg
    } else if data.starts_with(b"GIF8") {
        ImageFormat::Gif
    } else if data.len() >= 12 && &data[8..12] == b"WEBP" {
        ImageFormat::WebP
    } else {
        ImageFormat::Unknown
    }
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
        eprintln!(
            "[dreport] Image src data URI değil, atlanıyor: {}...",
            &src[..src.len().min(60)]
        );
        return;
    };

    use base64::Engine;
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(base64_part) else {
        eprintln!("[dreport] Image base64 decode hatası");
        return;
    };

    // Magic bytes ile format tespit et, krilla'nın native desteğini kullan
    let img_result = match detect_image_format(&decoded) {
        ImageFormat::Png => krilla::image::Image::from_png(decoded.into(), true),
        ImageFormat::Jpeg => krilla::image::Image::from_jpeg(decoded.into(), true),
        ImageFormat::Gif => krilla::image::Image::from_gif(decoded.into(), true),
        ImageFormat::WebP => krilla::image::Image::from_webp(decoded.into(), true),
        ImageFormat::Unknown => match decode_to_png(&decoded) {
            Some(png_data) => krilla::image::Image::from_png(png_data.into(), true),
            None => {
                eprintln!("[dreport] Image decode/re-encode hatası");
                return;
            }
        },
    };

    let Ok(img) = img_result else {
        eprintln!("[dreport] Image krilla embed hatası");
        return;
    };

    let Some(size) = Size::from_wh(w, h) else {
        return;
    };

    surface.push_transform(&Transform::from_translate(x, y));
    surface.draw_image(img, size);
    surface.pop();
}

#[allow(clippy::too_many_arguments)]
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

    let png_result = crate::barcode_gen::generate_barcode_png(
        format,
        value,
        w_px,
        h_px,
        include_text,
        Some(font_data),
    );

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

#[allow(clippy::too_many_arguments)]
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
    use crate::chart_layout::{
        color_at, compute_bar_layout, compute_chart_layout, compute_legend, compute_line_layout,
        compute_pie_layout, format_value,
    };

    let base_x_mm: f64 = (x / MM_TO_PT) as f64;
    let base_y_mm: f64 = (y / MM_TO_PT) as f64;
    let w_mm: f64 = (w / MM_TO_PT) as f64;
    let h_mm: f64 = (h / MM_TO_PT) as f64;

    // Background
    chart_rect(
        surface,
        base_x_mm,
        base_y_mm,
        w_mm,
        h_mm,
        parse_color(data.background_color.as_deref().unwrap_or("#FFFFFF")),
    );

    let cl = compute_chart_layout(data, w_mm, h_mm, base_x_mm, base_y_mm);

    // Title
    if let Some(ref title) = cl.title {
        let color = parse_color(&title.color);
        let font = fonts.get(None, Some("bold"), None);
        if let Some(f) = font {
            surface.set_fill(Some(fill_from_color(color)));
            surface.set_stroke(None);
            let fs_pt = pt(title.font_size);
            let (tw, _) = measurer.measure(&title.text, None, fs_pt, Some("bold"), None);
            let tx = match title.align.as_str() {
                "left" => pt(title.x),
                "right" => pt(title.x) - tw,
                _ => pt(title.x) - tw / 2.0,
            };
            let ty = pt(title.y);
            surface.draw_text(
                Point::from_xy(tx, ty),
                f.clone(),
                fs_pt,
                &title.text,
                false,
                TextDirection::Auto,
            );
        }
    }

    use dreport_core::models::ChartType;
    match data.chart_type {
        ChartType::Bar => {
            let bl = compute_bar_layout(data, &cl);
            render_chart_y_axis(surface, &bl.y_axis, fonts, measurer);
            for bar in &bl.bars {
                let color = parse_color(color_at(&cl.palette, bar.color_idx));
                chart_rect(surface, bar.x, bar.y, bar.w, bar.h, color);
                if bl.show_labels {
                    if bl.stacked {
                        if bar.value > 0.0 {
                            let label = format_value(bar.value);
                            chart_text_centered(
                                surface,
                                bar.label_x,
                                bar.label_y,
                                &label,
                                bl.label_font,
                                &bl.label_color,
                                fonts,
                                measurer,
                            );
                        }
                    } else {
                        let label = format_value(bar.value);
                        chart_text_centered(
                            surface,
                            bar.label_x,
                            bar.label_y,
                            &label,
                            bl.label_font,
                            &bl.label_color,
                            fonts,
                            measurer,
                        );
                    }
                }
            }
            render_chart_x_labels(surface, &bl.x_labels, fonts, measurer);
            let ac = parse_color("#9CA3AF");
            chart_line_seg(
                surface,
                bl.x_axis_x1,
                bl.x_axis_y,
                bl.x_axis_x2,
                bl.x_axis_y,
                ac,
                0.8,
            );
        }
        ChartType::Line => {
            let ll = compute_line_layout(data, &cl);
            render_chart_y_axis(surface, &ll.y_axis, fonts, measurer);
            for series_layout in &ll.series {
                let color = parse_color(color_at(&cl.palette, series_layout.color_idx));
                // Polyline
                let points: Vec<(f64, f64)> =
                    series_layout.points.iter().map(|p| (p.x, p.y)).collect();
                surface.set_fill(None);
                surface.set_stroke(Some(Stroke {
                    paint: color.into(),
                    width: pt(ll.line_width),
                    opacity: NormalizedF32::ONE,
                    ..Default::default()
                }));
                let path = {
                    let mut pb = PathBuilder::new();
                    for (i, (lx, ly)) in points.iter().enumerate() {
                        if i == 0 {
                            pb.move_to(pt(*lx), pt(*ly));
                        } else {
                            pb.line_to(pt(*lx), pt(*ly));
                        }
                    }
                    pb.finish()
                };
                if let Some(p) = path {
                    surface.draw_path(&p);
                }

                // Points
                if ll.show_points {
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
                        if let Some(p) = circle {
                            surface.draw_path(&p);
                        }
                    }
                }

                // Value labels
                if ll.show_labels {
                    for lp in &series_layout.points {
                        let label = format_value(lp.value);
                        chart_text_centered(
                            surface,
                            lp.x,
                            lp.y - 1.5,
                            &label,
                            ll.label_font,
                            &ll.label_color,
                            fonts,
                            measurer,
                        );
                    }
                }
            }
            render_chart_x_labels(surface, &ll.x_labels, fonts, measurer);
            let ac = parse_color("#9CA3AF");
            chart_line_seg(
                surface,
                ll.x_axis_x1,
                ll.x_axis_y,
                ll.x_axis_x2,
                ll.x_axis_y,
                ac,
                0.8,
            );
        }
        ChartType::Pie => {
            let pl = compute_pie_layout(data, &cl);
            for slice in &pl.slices {
                let color = parse_color(color_at(&cl.palette, slice.color_idx));
                surface.set_fill(Some(fill_from_color(color)));
                surface.set_stroke(Some(Stroke {
                    paint: rgb::Color::new(255, 255, 255).into(),
                    width: 0.8,
                    opacity: NormalizedF32::ONE,
                    ..Default::default()
                }));
                let path = build_arc_path(
                    pl.cx,
                    pl.cy,
                    pl.radius,
                    pl.inner_radius,
                    slice.start_angle,
                    slice.end_angle,
                );
                if let Some(p) = path {
                    surface.draw_path(&p);
                }

                if pl.show_labels {
                    let pct = (slice.fraction * 100.0).round();
                    let label = format!("{}%", pct);
                    chart_text_centered(
                        surface,
                        slice.label_x,
                        slice.label_y,
                        &label,
                        pl.label_font,
                        &pl.label_color,
                        fonts,
                        measurer,
                    );
                }

                if pl.show_cat_labels && !slice.cat_label_text.is_empty() {
                    chart_line_seg(
                        surface,
                        slice.leader_start_x,
                        slice.leader_start_y,
                        slice.leader_end_x,
                        slice.leader_end_y,
                        parse_color("#999999"),
                        0.5,
                    );
                    if slice.cat_label_anchor_end {
                        chart_text_end(
                            surface,
                            slice.cat_label_x,
                            slice.cat_label_y,
                            &slice.cat_label_text,
                            2.5,
                            "#555555",
                            fonts,
                            measurer,
                        );
                    } else {
                        chart_text_start(
                            surface,
                            slice.cat_label_x,
                            slice.cat_label_y,
                            &slice.cat_label_text,
                            2.5,
                            "#555555",
                            fonts,
                            measurer,
                        );
                    }
                }
            }
        }
    }

    // Legend render
    if cl.legend_show {
        let legend = compute_legend(data, &cl, base_x_mm, base_y_mm, w_mm, h_mm);
        for item in &legend.items {
            let color = parse_color(color_at(&cl.palette, item.color_idx));
            chart_rect(
                surface,
                item.swatch_x,
                item.swatch_y,
                legend.swatch_size,
                legend.swatch_size,
                color,
            );
            chart_text_start(
                surface,
                item.text_x,
                item.text_y,
                &item.name,
                legend.font_size,
                "#666666",
                fonts,
                measurer,
            );
        }
    }

    // Axis labels
    let is_pie = matches!(data.chart_type, ChartType::Pie);
    if !is_pie {
        if let Some(ref x_label) = data.x_label {
            let lx = cl.plot_x + cl.plot_w / 2.0;
            let ly = base_y_mm + h_mm - 2.0;
            chart_text_centered(surface, lx, ly, x_label, 2.8, "#666666", fonts, measurer);
        }
        if let Some(ref y_label) = data.y_label {
            let lx = base_x_mm + 3.0;
            let ly = cl.plot_y + cl.plot_h / 2.0;
            surface.push_transform(&Transform::from_translate(pt(lx), pt(ly)));
            surface.push_transform(&Transform::from_row(0.0, -1.0, 1.0, 0.0, 0.0, 0.0));
            chart_text_centered(surface, 0.0, 0.0, y_label, 2.8, "#666666", fonts, measurer);
            surface.pop();
            surface.pop();
        }
    }
}

/// mm degerlerini pt'ye cevirip rect ciz
fn chart_rect(
    surface: &mut krilla::surface::Surface<'_>,
    rx: f64,
    ry: f64,
    rw: f64,
    rh: f64,
    color: rgb::Color,
) {
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

fn chart_line_seg(
    surface: &mut krilla::surface::Surface<'_>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: rgb::Color,
    width: f32,
) {
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
#[allow(clippy::too_many_arguments)]
fn chart_text_centered(
    surface: &mut krilla::surface::Surface<'_>,
    cx_mm: f64,
    cy_mm: f64,
    text: &str,
    font_size_mm: f64,
    color_hex: &str,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None, None);
    let Some(f) = font else {
        return;
    };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    let (tw, _) = measurer.measure(text, None, fs_pt, None, None);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(cx_mm) - tw / 2.0, pt(cy_mm)),
        f.clone(),
        fs_pt,
        text,
        false,
        TextDirection::Auto,
    );
}

/// Chart icin metin ciz — end-aligned (sag hizali)
#[allow(clippy::too_many_arguments)]
fn chart_text_end(
    surface: &mut krilla::surface::Surface<'_>,
    right_x_mm: f64,
    cy_mm: f64,
    text: &str,
    font_size_mm: f64,
    color_hex: &str,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None, None);
    let Some(f) = font else {
        return;
    };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    let (tw, _) = measurer.measure(text, None, fs_pt, None, None);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(right_x_mm) - tw, pt(cy_mm)),
        f.clone(),
        fs_pt,
        text,
        false,
        TextDirection::Auto,
    );
}

/// Chart icin metin ciz — start-aligned (sol hizali)
#[allow(clippy::too_many_arguments)]
fn chart_text_start(
    surface: &mut krilla::surface::Surface<'_>,
    x_mm: f64,
    cy_mm: f64,
    text: &str,
    font_size_mm: f64,
    color_hex: &str,
    fonts: &FontCollection,
    _measurer: &mut TextMeasurer,
) {
    let font = fonts.get(None, None, None);
    let Some(f) = font else {
        return;
    };
    let color = parse_color(color_hex);
    let fs_pt = pt(font_size_mm);
    surface.set_fill(Some(fill_from_color(color)));
    surface.set_stroke(None);
    surface.draw_text(
        Point::from_xy(pt(x_mm), pt(cy_mm)),
        f.clone(),
        fs_pt,
        text,
        false,
        TextDirection::Auto,
    );
}

/// Y-axis grid + value labels — consumes shared YAxisLayout
fn render_chart_y_axis(
    surface: &mut krilla::surface::Surface<'_>,
    y_axis: &crate::chart_layout::YAxisLayout,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    for tick in &y_axis.ticks {
        chart_text_end(
            surface,
            y_axis.axis_x - 1.5,
            tick.y + 0.8,
            &tick.label,
            2.3,
            "#666666",
            fonts,
            measurer,
        );
        if y_axis.show_grid {
            let gc = parse_color(&y_axis.grid_color);
            chart_line_seg(
                surface,
                y_axis.axis_x,
                tick.y,
                y_axis.grid_end_x,
                tick.y,
                gc,
                0.4,
            );
        }
    }
    // Y axis line
    let ac = parse_color("#9CA3AF");
    chart_line_seg(
        surface,
        y_axis.axis_x,
        y_axis.axis_y_start,
        y_axis.axis_x,
        y_axis.axis_y_end,
        ac,
        0.8,
    );
}

/// X-axis category labels — consumes shared XLabelLayout
fn render_chart_x_labels(
    surface: &mut krilla::surface::Surface<'_>,
    x_labels: &crate::chart_layout::XLabelLayout,
    fonts: &FontCollection,
    measurer: &mut TextMeasurer,
) {
    let angle = x_labels.rotate_angle;
    for label in &x_labels.labels {
        if angle > 0.0 {
            surface.push_transform(&Transform::from_translate(pt(label.x), pt(label.y)));
            let angle_rad = (angle as f32).to_radians();
            let c = angle_rad.cos();
            let s = angle_rad.sin();
            surface.push_transform(&Transform::from_row(c, -s, s, c, 0.0, 0.0));
            chart_text_end(
                surface,
                0.0,
                0.0,
                &label.text,
                2.2,
                "#666666",
                fonts,
                measurer,
            );
            surface.pop();
            surface.pop();
        } else {
            chart_text_centered(
                surface,
                label.x,
                label.y,
                &label.text,
                2.5,
                "#666666",
                fonts,
                measurer,
            );
        }
    }
}

/// Arc path olustur — pie/donut dilimi (mm cinsinden, pt'ye cevrilir)
fn build_arc_path(
    cx: f64,
    cy: f64,
    radius: f64,
    inner_r: f64,
    start: f64,
    end: f64,
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
fn approximate_arc(pb: &mut PathBuilder, cx: f64, cy: f64, r: f64, start: f64, end: f64) {
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
            page: PageSettings {
                width: 210.0,
                height: 297.0,
            },
            fonts: vec!["Noto Sans".to_string()],
            header: None,
            footer: None,
            format_config: None,
            locale: None,
            root: ContainerElement {
                base: ElementBase::flow("root".to_string(), SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                }),
                direction: "column".to_string(),
                gap: 5.0,
                padding: Padding {
                    top: 15.0,
                    right: 15.0,
                    bottom: 15.0,
                    left: 15.0,
                },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::StaticText(StaticTextElement {
                        base: ElementBase::flow("title".to_string(), SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        }),
                        style: TextStyle {
                            font_size: Some(18.0),
                            font_weight: Some("bold".to_string()),
                            ..Default::default()
                        },
                        content: "FATURA".to_string(),
                    }),
                    TemplateElement::Line(LineElement {
                        base: ElementBase::flow("line1".to_string(), SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        }),
                        style: LineStyle {
                            stroke_color: Some("#000000".to_string()),
                            stroke_width: Some(0.5),
                        },
                    }),
                    TemplateElement::Text(TextElement {
                        base: ElementBase::flow("firma".to_string(), SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        }),
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
        let layout = crate::compute_layout(&template, &data, &fonts).unwrap();
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

    // --- parse_color tests ---

    #[test]
    fn test_parse_color_6_digit_hex() {
        let c = parse_color("#FF8800");
        assert_eq!(c, rgb::Color::new(255, 136, 0));
    }

    #[test]
    fn test_parse_color_3_digit_hex() {
        let c = parse_color("#F80");
        assert_eq!(c, rgb::Color::new(255, 136, 0)); // F*17=255, 8*17=136, 0*17=0
    }

    #[test]
    fn test_parse_color_without_hash() {
        let c = parse_color("00FF00");
        assert_eq!(c, rgb::Color::new(0, 255, 0));
    }

    #[test]
    fn test_parse_color_black() {
        let c = parse_color("#000000");
        assert_eq!(c, rgb::Color::new(0, 0, 0));
    }

    #[test]
    fn test_parse_color_white() {
        let c = parse_color("#FFFFFF");
        assert_eq!(c, rgb::Color::new(255, 255, 255));
    }

    #[test]
    fn test_parse_color_invalid_length() {
        // Invalid length → defaults to (0,0,0)
        let c = parse_color("#ABCD");
        assert_eq!(c, rgb::Color::new(0, 0, 0));
    }

    #[test]
    fn test_parse_color_empty() {
        let c = parse_color("");
        assert_eq!(c, rgb::Color::new(0, 0, 0));
    }

    // --- build_rect_path tests ---

    #[test]
    fn test_build_rect_path_no_radius() {
        let path = build_rect_path(10.0, 20.0, 100.0, 50.0, 0.0);
        assert!(path.is_some(), "should produce valid rect path with no radius");
    }

    #[test]
    fn test_build_rect_path_with_radius() {
        let path = build_rect_path(0.0, 0.0, 100.0, 50.0, 5.0);
        assert!(path.is_some(), "should produce valid rounded rect path");
    }

    #[test]
    fn test_build_rect_path_radius_clamped() {
        // Radius larger than half the smaller dimension → should be clamped
        let path = build_rect_path(0.0, 0.0, 20.0, 10.0, 100.0);
        assert!(path.is_some(), "should clamp radius and produce valid path");
    }

    // --- build_ellipse_path tests ---

    #[test]
    fn test_build_ellipse_path() {
        let path = build_ellipse_path(10.0, 20.0, 60.0, 40.0);
        assert!(path.is_some(), "should produce valid ellipse path");
    }

    #[test]
    fn test_build_ellipse_path_circle() {
        // Equal width and height → circle
        let path = build_ellipse_path(0.0, 0.0, 50.0, 50.0);
        assert!(path.is_some(), "should produce valid circle path");
    }

    // --- mm/pt conversion tests ---

    #[test]
    fn test_mm_to_pt_conversion() {
        // 25.4mm = 72pt (1 inch)
        let result = mm(25.4);
        assert!((result - 72.0).abs() < 0.01, "25.4mm should be ~72pt, got {}", result);
    }

    #[test]
    fn test_mm_zero() {
        assert_eq!(mm(0.0), 0.0);
    }

    #[test]
    fn test_pt_conversion() {
        let result = pt(25.4);
        assert!((result - 72.0).abs() < 0.01);
    }

    // --- render_pdf integration with various element types ---

    #[test]
    fn test_render_pdf_with_line_element() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![ElementLayout {
                    id: "line1".to_string(),
                    x_mm: 15.0,
                    y_mm: 50.0,
                    width_mm: 180.0,
                    height_mm: 0.5,
                    element_type: "line".to_string(),
                    content: Some(ResolvedContent::Line),
                    style: ResolvedStyle {
                        stroke_color: Some("#FF0000".to_string()),
                        stroke_width: Some(1.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            }],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("should render line element");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_pdf_with_container_background() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![ElementLayout {
                    id: "box".to_string(),
                    x_mm: 20.0,
                    y_mm: 20.0,
                    width_mm: 170.0,
                    height_mm: 100.0,
                    element_type: "container".to_string(),
                    content: None,
                    style: ResolvedStyle {
                        background_color: Some("#E0E0E0".to_string()),
                        border_color: Some("#333333".to_string()),
                        border_width: Some(0.5),
                        border_radius: Some(3.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            }],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("should render container bg");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_pdf_with_shape_element() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![ElementLayout {
                    id: "shape1".to_string(),
                    x_mm: 50.0,
                    y_mm: 50.0,
                    width_mm: 40.0,
                    height_mm: 40.0,
                    element_type: "shape".to_string(),
                    content: Some(ResolvedContent::Shape {
                        shape_type: "ellipse".to_string(),
                    }),
                    style: ResolvedStyle {
                        background_color: Some("#3366FF".to_string()),
                        border_color: Some("#000000".to_string()),
                        border_width: Some(1.0),
                        ..Default::default()
                    },
                    children: vec![],
                }],
            }],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("should render shape element");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_pdf_with_checkbox() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![
                    ElementLayout {
                        id: "cb_checked".to_string(),
                        x_mm: 15.0,
                        y_mm: 15.0,
                        width_mm: 5.0,
                        height_mm: 5.0,
                        element_type: "checkbox".to_string(),
                        content: Some(ResolvedContent::Checkbox { checked: true }),
                        style: ResolvedStyle::default(),
                        children: vec![],
                    },
                    ElementLayout {
                        id: "cb_unchecked".to_string(),
                        x_mm: 15.0,
                        y_mm: 25.0,
                        width_mm: 5.0,
                        height_mm: 5.0,
                        element_type: "checkbox".to_string(),
                        content: Some(ResolvedContent::Checkbox { checked: false }),
                        style: ResolvedStyle::default(),
                        children: vec![],
                    },
                ],
            }],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("should render checkbox elements");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_pdf_empty_page() {
        let layout = LayoutResult {
            pages: vec![PageLayout {
                page_index: 0,
                width_mm: 210.0,
                height_mm: 297.0,
                elements: vec![],
            }],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("empty page should still render");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_pdf_multi_page() {
        let layout = LayoutResult {
            pages: vec![
                PageLayout {
                    page_index: 0,
                    width_mm: 210.0,
                    height_mm: 297.0,
                    elements: vec![ElementLayout {
                        id: "p1".to_string(),
                        x_mm: 15.0,
                        y_mm: 15.0,
                        width_mm: 180.0,
                        height_mm: 10.0,
                        element_type: "static_text".to_string(),
                        content: Some(ResolvedContent::Text { value: "Page 1".to_string() }),
                        style: ResolvedStyle { font_size: Some(12.0), ..Default::default() },
                        children: vec![],
                    }],
                },
                PageLayout {
                    page_index: 1,
                    width_mm: 210.0,
                    height_mm: 297.0,
                    elements: vec![ElementLayout {
                        id: "p2".to_string(),
                        x_mm: 15.0,
                        y_mm: 15.0,
                        width_mm: 180.0,
                        height_mm: 10.0,
                        element_type: "static_text".to_string(),
                        content: Some(ResolvedContent::Text { value: "Page 2".to_string() }),
                        style: ResolvedStyle { font_size: Some(12.0), ..Default::default() },
                        children: vec![],
                    }],
                },
            ],
        };
        let fonts = test_fonts();
        let pdf = render_pdf(&layout, &fonts).expect("multi-page should render");
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 200, "multi-page PDF should have reasonable size");
    }

    // --- detect_image_format tests ---

    #[test]
    fn test_detect_png() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_image_format(&data), ImageFormat::Png);
    }

    #[test]
    fn test_detect_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_image_format(&data), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_gif() {
        assert_eq!(detect_image_format(b"GIF89a..."), ImageFormat::Gif);
        assert_eq!(detect_image_format(b"GIF87a..."), ImageFormat::Gif);
    }

    #[test]
    fn test_detect_webp() {
        // RIFF____WEBP
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(detect_image_format(&data), ImageFormat::WebP);
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(
            detect_image_format(&[0x00, 0x01, 0x02]),
            ImageFormat::Unknown
        );
        assert_eq!(detect_image_format(&[]), ImageFormat::Unknown);
    }
}
