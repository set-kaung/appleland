use crate::app_state::AppState;
use axum::Router;
use axum::routing::get;

pub mod handler;
pub mod model;
pub mod store;

use handler::{get_category_performance, get_expiring_soon, get_product_performance};

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/ai/product-performance", get(get_product_performance))
        .route("/ai/by-category", get(get_category_performance))
        .route("/ai/expiring-soon", get(get_expiring_soon))
}
