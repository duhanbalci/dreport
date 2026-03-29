use axum::{Router, serve};
use dreport_layout::FontData;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod models;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Fontlar yukleniyor...");
    let fonts = Arc::new(load_fonts());
    println!("Fontlar yuklendi ({} font dosyasi)", fonts.len());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes::router())
        .layer(cors)
        .with_state(fonts);

    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("dreport backend listening on http://localhost:3001");
    serve(listener, app).await?;

    Ok(())
}

/// Proje fontlarını yükler (backend/fonts/ dizininden).
fn load_fonts() -> Vec<FontData> {
    let font_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts");
    let mut fonts = Vec::new();

    let entries = std::fs::read_dir(&font_dir).expect("backend/fonts dizini bulunamadi");
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ttf" || e == "otf") {
            let data = std::fs::read(&path).unwrap();
            let family = if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("Mono")
            {
                "Noto Sans Mono".to_string()
            } else {
                "Noto Sans".to_string()
            };
            fonts.push(FontData { family, data });
        }
    }
    fonts
}
