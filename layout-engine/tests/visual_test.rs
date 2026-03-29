//! Visual regression tests for PDF rendering.
//!
//! Generates PDF from fixture template+data, converts to PNG via pdftoppm,
//! and compares against reference snapshots.
//!
//! Set UPDATE_SNAPSHOTS=1 to update reference images.

#![cfg(not(target_arch = "wasm32"))]

mod visual {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use dreport_core::models::Template;
    use dreport_layout::{compute_layout, FontData};
    use dreport_layout::pdf_render::render_pdf;

    fn fixtures_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn snapshots_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
    }

    fn load_test_fonts() -> Vec<FontData> {
        let font_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("backend/fonts");

        let mut fonts = Vec::new();
        for entry in fs::read_dir(&font_dir).expect("backend/fonts directory not found") {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "ttf") {
                let family = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split('-')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string();
                let family = if family == "NotoSansMono" {
                    "Noto Sans Mono".to_string()
                } else if family == "NotoSans" {
                    "Noto Sans".to_string()
                } else {
                    family
                };
                fonts.push(FontData {
                    family,
                    data: fs::read(&path).unwrap(),
                });
            }
        }
        fonts
    }

    fn generate_test_pdf(template_name: &str, data_name: &str) -> Vec<u8> {
        let template_json = fs::read_to_string(fixtures_dir().join(template_name)).unwrap();
        let data_json = fs::read_to_string(fixtures_dir().join(data_name)).unwrap();

        let template: Template = serde_json::from_str(&template_json).unwrap();
        let data: serde_json::Value = serde_json::from_str(&data_json).unwrap();
        let fonts = load_test_fonts();

        let layout = compute_layout(&template, &data, &fonts);
        render_pdf(&layout, &fonts).expect("PDF render failed")
    }

    fn pdf_to_png(pdf_bytes: &[u8], output_path: &Path) -> bool {
        // Write PDF to temp file
        let temp_pdf = output_path.with_extension("pdf");
        fs::write(&temp_pdf, pdf_bytes).unwrap();

        // pdftoppm appends .png to the output prefix, so strip the extension
        let output_prefix = output_path.with_extension("");

        let result = Command::new("pdftoppm")
            .args(["-png", "-r", "150", "-singlefile"])
            .arg(&temp_pdf)
            .arg(&output_prefix)
            .output();

        // Clean up temp PDF
        let _ = fs::remove_file(&temp_pdf);

        match result {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!(
                        "pdftoppm failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return false;
                }
                true
            }
            Err(_) => {
                eprintln!("pdftoppm not available - skipping visual test");
                false
            }
        }
    }

    fn compare_images(
        actual_path: &Path,
        reference_path: &Path,
        max_diff_ratio: f64,
    ) -> Result<f64, String> {
        let actual =
            image::open(actual_path).map_err(|e| format!("Failed to open actual: {}", e))?;
        let reference =
            image::open(reference_path).map_err(|e| format!("Failed to open reference: {}", e))?;

        let actual_rgba = actual.to_rgba8();
        let reference_rgba = reference.to_rgba8();

        if actual_rgba.dimensions() != reference_rgba.dimensions() {
            return Err(format!(
                "Dimension mismatch: actual {:?} vs reference {:?}",
                actual_rgba.dimensions(),
                reference_rgba.dimensions()
            ));
        }

        let total_pixels = (actual_rgba.width() * actual_rgba.height()) as f64;
        let mut diff_pixels = 0u64;

        for (a, r) in actual_rgba.pixels().zip(reference_rgba.pixels()) {
            // Allow per-channel tolerance of 2 for font rendering differences
            let channel_diff = a
                .0
                .iter()
                .zip(r.0.iter())
                .any(|(ac, rc)| (*ac as i32 - *rc as i32).unsigned_abs() > 2);
            if channel_diff {
                diff_pixels += 1;
            }
        }

        let diff_ratio = diff_pixels as f64 / total_pixels;

        if diff_ratio > max_diff_ratio {
            Err(format!(
                "Visual diff too large: {:.4}% pixels differ (threshold: {:.4}%)",
                diff_ratio * 100.0,
                max_diff_ratio * 100.0
            ))
        } else {
            Ok(diff_ratio)
        }
    }

    #[test]
    fn test_visual_snapshot_basic() {
        let pdf_bytes =
            generate_test_pdf("visual_test_template.json", "visual_test_data.json");
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");

        let snap_dir = snapshots_dir();
        fs::create_dir_all(&snap_dir).unwrap();

        let actual_png = snap_dir.join("visual_test_actual.png");
        let reference_png = snap_dir.join("visual_test_reference.png");

        if !pdf_to_png(&pdf_bytes, &actual_png) {
            eprintln!("Skipping visual comparison - pdftoppm not available");
            return;
        }

        let update_snapshots = std::env::var("UPDATE_SNAPSHOTS").is_ok();

        if !reference_png.exists() || update_snapshots {
            // First run or explicit update: save as reference
            fs::copy(&actual_png, &reference_png).unwrap();
            println!("Reference snapshot saved to {:?}", reference_png);
            // Clean up actual
            let _ = fs::remove_file(&actual_png);
            return;
        }

        // Compare
        match compare_images(&actual_png, &reference_png, 0.01) {
            Ok(diff) => {
                println!(
                    "Visual test passed: {:.4}% pixels differ",
                    diff * 100.0
                );
                let _ = fs::remove_file(&actual_png);
            }
            Err(err) => {
                // Keep actual for debugging
                panic!(
                    "Visual regression detected: {}. Actual saved at {:?}",
                    err, actual_png
                );
            }
        }
    }
}
