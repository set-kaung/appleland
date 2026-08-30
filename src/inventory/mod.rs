use crate::app_state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub mod handler;
pub mod model;
pub mod store;

use handler::{get_product_inventory_records, insert_inventory_record};

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/{product_id}", post(insert_inventory_record))
        .route("/{product_id}", get(get_product_inventory_records))
}
