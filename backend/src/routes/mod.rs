mod fonts;
mod health;
mod render;

use axum::Router;
use dreport_service::DreportService;
use std::sync::Arc;

pub type AppState = Arc<DreportService>;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(render::router())
        .merge(fonts::router())
}
