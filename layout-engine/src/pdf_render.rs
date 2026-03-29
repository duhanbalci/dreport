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

fn fill_from_color(color: rgb::Color) -> Fill {
    Fill {
        paint: color.into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    }
}

/// Font koleksiyonu — family + weight + italic → KrillaFont mapping
struct FontCollection {
    /// (family_lower, is_bold, is_italic) → KrillaFont
    fonts: HashMap<(String, bool, bool), KrillaFont>,
    /// Fallback font (ilk yüklenen regular)
    default: Option<KrillaFont>,
}

impl FontCollection {
    fn new(font_data: &[FontData]) -> Self {
        let mut fonts = HashMap::new();
        let mut default = None;

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

            fonts.insert((family_lower.clone(), is_bold, is_italic), font);
        }

        // Hiç regular bulamadıysak ilk font'u default yap
        if default.is_none() {
            if let Some(fd) = font_data.first() {
                default = KrillaFont::new(krilla::Data::from(fd.data.clone()), 0);
            }
        }

        Self { fonts, default }
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

    let Some(ref content) = el.content else {
        return;
    };

    match content {
        ResolvedContent::Text { value } => {
            render_text(surface, x, y, w, h, value, &el.style, fonts, measurer);
        }
        ResolvedContent::Line => {
            render_line(surface, x, y, w, &el.style);
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
        ResolvedContent::Barcode { format, value } => {
            render_barcode(surface, x, y, w, h, format, value, &el.style, font_data);
        }
    }
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

    // Fill
    if let Some(ref bg) = style.background_color {
        surface.set_fill(Some(fill_from_color(parse_color(bg))));
    } else {
        surface.set_fill(None);
    }

    // Stroke
    if has_border {
        let border_color = parse_color(style.border_color.as_deref().unwrap_or("#000000"));
        let border_width = mm(style.border_width.unwrap_or(0.5));
        surface.set_stroke(Some(Stroke {
            paint: border_color.into(),
            width: border_width,
            opacity: NormalizedF32::ONE,
            ..Default::default()
        }));
    } else {
        surface.set_stroke(None);
    }

    let rect_path = {
        let mut pb = PathBuilder::new();
        if let Some(rect) = krilla::geom::Rect::from_xywh(x, y, w, h) {
            pb.push_rect(rect);
        }
        pb.finish()
    };

    if let Some(path) = rect_path {
        surface.draw_path(&path);
    }

    // Reset
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

    // Text baseline: y + ascent (yaklaşık font_size * 0.8)
    let baseline_y = y + font_size * 0.8;

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

fn render_line(
    surface: &mut krilla::surface::Surface<'_>,
    x: f32,
    y: f32,
    w: f32,
    style: &ResolvedStyle,
) {
    let stroke_color = style
        .stroke_color
        .as_deref()
        .map(parse_color)
        .unwrap_or(rgb::Color::new(0, 0, 0));
    let stroke_width = mm(style.stroke_width.unwrap_or(0.5));

    surface.set_fill(None);
    surface.set_stroke(Some(Stroke {
        paint: stroke_color.into(),
        width: stroke_width,
        opacity: NormalizedF32::ONE,
        ..Default::default()
    }));

    let line_y = y + stroke_width / 2.0;
    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(x, line_y);
        pb.line_to(x + w, line_y);
        pb.finish()
    };

    if let Some(p) = path {
        surface.draw_path(&p);
    }

    surface.set_stroke(None);
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
