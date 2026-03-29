use axum::{Router, serve};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod models;
mod routes;
mod typst_engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Fontları bir kez yükle — tüm request'lerde paylaşılacak
    println!("Fontlar yukleniyor...");
    let fonts = Arc::new(typst_engine::fonts::load_fonts());
    println!("Fontlar yuklendi ({} font)", fonts.fonts.len());

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
