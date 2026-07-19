use crate::app_state::AppState;
use axum::routing::post;
use axum::Router;

pub mod handler;
pub mod model;
pub mod store;

use handler::insert_inventory;

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/{product_id}", post(insert_inventory))
}

