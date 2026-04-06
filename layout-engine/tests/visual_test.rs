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
    use dreport_layout::{compute_layout, FontData, ResolvedContent};
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
                let data = fs::read(&path).unwrap();
                if let Some(fd) = FontData::from_bytes(data) {
                    fonts.push(fd);
                }
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

        let layout = compute_layout(&template, &data, &fonts).unwrap();
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

    fn run_visual_test(template_file: &str, data_file: &str, test_name: &str) {
        let pdf_bytes = generate_test_pdf(template_file, data_file);
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");

        let snap_dir = snapshots_dir();
        fs::create_dir_all(&snap_dir).unwrap();

        let actual_png = snap_dir.join(format!("{}_actual.png", test_name));
        let reference_png = snap_dir.join(format!("{}_reference.png", test_name));

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
                    "Visual test [{}] passed: {:.4}% pixels differ",
                    test_name,
                    diff * 100.0
                );
                let _ = fs::remove_file(&actual_png);
            }
            Err(err) => {
                // Keep actual for debugging
                panic!(
                    "Visual regression [{}]: {}. Actual saved at {:?}",
                    test_name, err, actual_png
                );
            }
        }
    }

    /// SVG'yi standalone HTML'e sar — chart'ın HTML render'ını görmek icin
    fn generate_chart_svg_html(template_file: &str, data_file: &str, output_path: &Path) {
        let template_json = fs::read_to_string(fixtures_dir().join(template_file)).unwrap();
        let data_json = fs::read_to_string(fixtures_dir().join(data_file)).unwrap();

        let template: Template = serde_json::from_str(&template_json).unwrap();
        let data: serde_json::Value = serde_json::from_str(&data_json).unwrap();
        let fonts = load_test_fonts();

        let layout = compute_layout(&template, &data, &fonts).unwrap();

        let mut html = String::from("<!DOCTYPE html><html><head><style>body{margin:20px;font-family:sans-serif;background:#f5f5f5}.chart-box{margin:10px 0;background:white;box-shadow:0 1px 3px rgba(0,0,0,.1)}</style></head><body><h2>Chart SVG Preview (HTML render)</h2>");

        for page in &layout.pages {
            for el in &page.elements {
                if let Some(ResolvedContent::Chart { svg, .. }) = &el.content {
                    html.push_str(&format!(
                        "<div class='chart-box' style='width:{}mm;height:{}mm'>{}</div>",
                        el.width_mm, el.height_mm, svg
                    ));
                }
            }
        }

        html.push_str("</body></html>");
        fs::write(output_path, html).unwrap();
    }

    /// Cross-renderer reference PNG output directory
    fn cross_renderer_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("frontend/tests/visual/cross-renderer-refs")
    }

    /// Generates PDF→PNG references for cross-renderer comparison with HTML render.
    /// Run explicitly: cargo test -p dreport-layout --test visual_test -- generate_cross_renderer --ignored
    #[test]
    #[ignore]
    fn generate_cross_renderer_refs() {
        let fixtures = [
            ("visual_test_template.json", "visual_test_data.json", "visual_test"),
            ("chart_test_template.json", "chart_test_data.json", "chart_test"),
            ("comprehensive_test_template.json", "comprehensive_test_data.json", "comprehensive_test"),
        ];

        let out_dir = cross_renderer_dir();
        fs::create_dir_all(&out_dir).unwrap();

        for (template_file, data_file, name) in &fixtures {
            let pdf_bytes = generate_test_pdf(template_file, data_file);
            assert!(!pdf_bytes.is_empty(), "PDF should not be empty for {}", name);

            let png_path = out_dir.join(format!("{}.png", name));
            if !pdf_to_png(&pdf_bytes, &png_path) {
                panic!("pdftoppm failed for {} — install poppler-utils", name);
            }
            println!("Cross-renderer reference: {:?}", png_path);
        }
    }

    #[test]
    fn test_visual_snapshot_basic() {
        run_visual_test("visual_test_template.json", "visual_test_data.json", "visual_test");
    }

    #[test]
    fn test_visual_snapshot_charts() {
        let pdf_bytes = generate_test_pdf("chart_test_template.json", "chart_test_data.json");
        assert!(!pdf_bytes.is_empty(), "Chart PDF should not be empty");

        let snap_dir = snapshots_dir();
        fs::create_dir_all(&snap_dir).unwrap();

        // PDF ciktisini kaydet (inceleme icin)
        let pdf_path = snap_dir.join("chart_test.pdf");
        fs::write(&pdf_path, &pdf_bytes).unwrap();
        println!("Chart PDF saved to {:?}", pdf_path);

        // SVG HTML ciktisini kaydet (karsilastirma icin)
        let html_path = snap_dir.join("chart_test_svg.html");
        generate_chart_svg_html("chart_test_template.json", "chart_test_data.json", &html_path);
        println!("Chart SVG HTML saved to {:?}", html_path);

        // Visual regression test
        run_visual_test("chart_test_template.json", "chart_test_data.json", "chart_test");
    }
}
