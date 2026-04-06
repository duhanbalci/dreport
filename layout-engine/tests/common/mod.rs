use dreport_layout::FontData;

pub fn load_test_fonts() -> Vec<FontData> {
    let font_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("backend/fonts");

    let mut fonts = Vec::new();
    for entry in std::fs::read_dir(&font_dir).expect("backend/fonts directory not found") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ttf") {
            let data = std::fs::read(&path).unwrap();
            if let Some(fd) = FontData::from_bytes(data) {
                fonts.push(fd);
            }
        }
    }
    fonts
}
