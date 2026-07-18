use crate::app_state::AppState;
use axum::Router;

// use crate::inventory::handler::update_inventory;
pub mod handler;
mod model;
pub mod store;

pub fn route_admin() -> Router<AppState> {
    // Router::new().route("/{product_id}", put(update_inventory))
    Router::new()
}

