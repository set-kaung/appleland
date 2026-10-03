use crate::app_state::AppState;
use axum::Router;
use axum::routing::get;

pub mod handler;
pub mod model;
pub mod store;

use handler::get_product_performance;

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/ai/product-performance", get(get_product_performance))
}
