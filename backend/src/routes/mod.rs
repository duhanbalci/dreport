mod fonts;
mod health;
mod render;

use axum::Router;
use std::sync::Arc;

use crate::font_registry::FontRegistry;

pub fn router() -> Router<Arc<FontRegistry>> {
    Router::new()
        .merge(health::router())
        .merge(render::router())
        .merge(fonts::router())
}
