use axum::{Router, serve};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod font_registry;
mod models;
mod routes;

use font_registry::FontRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Font registry başlatılıyor...");
    let registry = Arc::new(FontRegistry::new());

    let family_count =
        dreport_layout::font_provider::FontProvider::list_families(registry.as_ref()).len();
    println!("Font registry hazır ({} font ailesi)", family_count);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes::router())
        .layer(cors)
        .with_state(registry);

    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("dreport backend listening on http://localhost:3001");
    serve(listener, app).await?;

    Ok(())
}
