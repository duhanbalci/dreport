mod health;
mod render;

use axum::Router;
use std::sync::Arc;
use typst_kit::fonts::Fonts;

pub fn router() -> Router<Arc<Fonts>> {
    Router::new()
        .merge(health::router())
        .merge(render::router())
}
