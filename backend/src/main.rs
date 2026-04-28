use dreport_backend::{app, build_router};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Arc::new(app::build_service()?);
    println!(
        "dreport-service hazır ({} font ailesi)",
        service.font_family_count()
    );

    let app = build_router(service);
    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("dreport backend listening on http://localhost:3001");
    axum::serve(listener, app).await?;

    Ok(())
}
