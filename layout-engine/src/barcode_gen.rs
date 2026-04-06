//! Barcode/QR code üretimi — rxing ile.
//! Hem native hem WASM'da derlenir.
//! Text rendering: cosmic-text (font varsa) veya bitmap fallback.

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use rxing::{BarcodeFormat, EncodeHints, Writer};

use crate::FontData;

/// dreport format string → rxing BarcodeFormat
fn to_rxing_format(format: &str) -> Result<BarcodeFormat, String> {
    match format {
        "qr" => Ok(BarcodeFormat::QR_CODE),
        "ean13" => Ok(BarcodeFormat::EAN_13),
        "ean8" => Ok(BarcodeFormat::EAN_8),
        "code128" => Ok(BarcodeFormat::CODE_128),
        "code39" => Ok(BarcodeFormat::CODE_39),
        _ => Err(format!("Desteklenmeyen barcode formatı: {format}")),
    }
}

/// Barcode üretim sonucu — ham grayscale pixel verisi
pub struct BarcodePixels {
    /// Grayscale pixel verileri (0=siyah, 255=beyaz), row-major
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Herhangi bir barcode formatında ham pixel verisi üret.
/// `include_text`: true ise lineer barkodların altına değer yazılır (QR için etkisiz).
/// `font_data`: Verilirse cosmic-text ile güzel font rendering, yoksa bitmap fallback.
pub fn generate_barcode_pixels(
    format: &str,
    value: &str,
    width_px: u32,
    height_px: u32,
    include_text: bool,
    font_data: Option<&[FontData]>,
) -> Result<BarcodePixels, String> {
    if value.is_empty() {
        return Err("Boş barcode değeri".to_string());
    }

    let bc_format = to_rxing_format(format)?;
    let is_qr = bc_format == BarcodeFormat::QR_CODE;

    // QR kod her zaman kare olmalı
    let (req_w, req_h) = if is_qr {
        let side = width_px.min(height_px);
        (side, side)
    } else {
        (width_px, height_px)
    };

    // Metin alanı hesapla (QR hariç, include_text true ise)
    let text_area_h = if !is_qr && include_text {
        (req_h / 5).clamp(16, 48)
    } else {
        0
    };
    let bar_h = req_h - text_area_h;

    let mut hints = EncodeHints {
        Margin: Some("1".to_string()),
        ..Default::default()
    };
    if is_qr {
        hints.ErrorCorrection = Some("M".to_string());
    }

    let writer = rxing::MultiFormatWriter;
    let matrix = writer
        .encode_with_hints(value, &bc_format, req_w as i32, bar_h as i32, &hints)
        .map_err(|e| format!("Barcode encode hatası ({format}): {e}"))?;

    let mat_w = matrix.width() as u32;
    let mat_h = matrix.height() as u32;

    // Çıktı boyutu: bar matrisi + metin alanı
    let out_w = mat_w;
    let out_h = mat_h + text_area_h;
    let mut pixels = vec![255u8; (out_w * out_h) as usize];

    // Bar matrisini çiz
    for y in 0..mat_h {
        for x in 0..mat_w {
            if matrix.get(x, y) {
                pixels[(y * out_w + x) as usize] = 0;
            }
        }
    }

    // Metin rendering
    if text_area_h > 0 && !is_qr {
        render_text_cosmic(
            &mut pixels,
            out_w,
            out_h,
            mat_h,
            text_area_h,
            value,
            font_data,
        );
    }

    Ok(BarcodePixels {
        pixels,
        width: out_w,
        height: out_h,
    })
}

/// cosmic-text ile metin render et — gerçek font rendering
fn render_text_cosmic(
    pixels: &mut [u8],
    img_w: u32,
    img_h: u32,
    text_y: u32,
    text_h: u32,
    text: &str,
    font_data: Option<&[FontData]>,
) {
    let mut font_system = FontSystem::new_with_locale_and_db(
        "tr-TR".to_string(),
        cosmic_text::fontdb::Database::new(),
    );

    match font_data {
        Some(fonts) if !fonts.is_empty() => {
            for f in fonts {
                font_system.db_mut().load_font_data(f.data.clone());
            }
        }
        _ => return, // Font yoksa metin render edemeyiz
    }

    // Font boyutunu text alanına göre ayarla (px cinsinden)
    let font_size_px = (text_h as f32 * 0.7).max(10.0);
    let line_height_px = font_size_px * 1.2;
    let metrics = Metrics::new(font_size_px, line_height_px);

    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(img_w as f32), Some(text_h as f32));

    let attrs = Attrs::new().family(Family::SansSerif);
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut font_system, false);

    let mut swash_cache = SwashCache::new();

    // Text genişliğini hesapla (ortalama için)
    let mut text_width: f32 = 0.0;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            let end = glyph.x + glyph.w;
            if end > text_width {
                text_width = end;
            }
        }
    }

    // Ortalama offset
    let offset_x = if (text_width as u32) < img_w {
        ((img_w as f32 - text_width) / 2.0) as i32
    } else {
        0
    };
    let offset_y = text_y as i32 + ((text_h as f32 - line_height_px) / 2.0).max(0.0) as i32;

    // Glyph'leri pixel buffer'a çiz
    for run in buffer.layout_runs() {
        let line_y = offset_y + run.line_y as i32;

        for glyph in run.glyphs.iter() {
            let physical = glyph.physical((offset_x as f32, line_y as f32), 1.0);

            let Some(image) = swash_cache.get_image_uncached(&mut font_system, physical.cache_key)
            else {
                continue;
            };

            let gx = physical.x + image.placement.left;
            let gy = physical.y - image.placement.top;
            let gw = image.placement.width as i32;
            let gh = image.placement.height as i32;

            for row in 0..gh {
                for col in 0..gw {
                    let px = gx + col;
                    let py = gy + row;
                    if px < 0 || py < 0 || px >= img_w as i32 || py >= img_h as i32 {
                        continue;
                    }

                    let src_idx = (row * gw + col) as usize;
                    if src_idx >= image.data.len() {
                        continue;
                    }

                    let alpha = image.data[src_idx];
                    if alpha == 0 {
                        continue;
                    }

                    let dst_idx = (py as u32 * img_w + px as u32) as usize;
                    if dst_idx >= pixels.len() {
                        continue;
                    }

                    // Alpha blending: beyaz arka plan üzerine siyah metin
                    let bg = pixels[dst_idx] as f32;
                    let a = alpha as f32 / 255.0;
                    pixels[dst_idx] = (bg * (1.0 - a)) as u8;
                }
            }
        }
    }
}

/// PNG bytes olarak barcode üret (sadece native).
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_barcode_png(
    format: &str,
    value: &str,
    width_px: u32,
    height_px: u32,
    include_text: bool,
    font_data: Option<&[FontData]>,
) -> Result<Vec<u8>, String> {
    let result =
        generate_barcode_pixels(format, value, width_px, height_px, include_text, font_data)?;

    let img = image::GrayImage::from_raw(result.width, result.height, result.pixels)
        .ok_or_else(|| "Pixel buffer boyutu uyumsuz".to_string())?;

    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::L8,
    )
    .map_err(|e| format!("PNG encode hatası: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_is_square() {
        let result =
            generate_barcode_pixels("qr", "https://example.com", 300, 200, false, None).unwrap();
        assert_eq!(result.width, result.height);
    }

    #[test]
    fn test_ean13_with_text() {
        let result =
            generate_barcode_pixels("ean13", "5901234123457", 300, 100, true, None).unwrap();
        assert!(result.width > 0);
        assert!(result.height > 0);
    }

    #[test]
    fn test_ean13_without_text() {
        let result =
            generate_barcode_pixels("ean13", "5901234123457", 300, 100, false, None).unwrap();
        assert!(result.width > 0);
        assert!(result.height > 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_ean13_with_font_rendering() {
        let fonts = crate::text_measure::load_test_fonts();
        let result =
            generate_barcode_pixels("ean13", "5901234123457", 400, 150, true, Some(&fonts))
                .unwrap();
        assert!(result.width > 0);
        assert!(result.height > 0);
        // Metin alanında siyah pikseller olmalı (font rendering çalıştı)
        let text_start = (result.height - result.height / 5) * result.width;
        let text_pixels = &result.pixels[text_start as usize..];
        assert!(
            text_pixels.iter().any(|&p| p < 128),
            "Font rendering metin üretmeli"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_qr_png() {
        let png = generate_barcode_png("qr", "https://example.com", 200, 200, false, None).unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_ean13_png_with_text() {
        let fonts = crate::text_measure::load_test_fonts();
        let png =
            generate_barcode_png("ean13", "5901234123457", 400, 150, true, Some(&fonts)).unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_code128_png() {
        let png = generate_barcode_png("code128", "ABC-123", 300, 80, true, None).unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }
}
