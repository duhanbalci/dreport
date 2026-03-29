mod health;
mod render;

use axum::Router;
use dreport_layout::FontData;
use std::sync::Arc;

pub fn router() -> Router<Arc<Vec<FontData>>> {
    Router::new()
        .merge(health::router())
        .merge(render::router())
}
