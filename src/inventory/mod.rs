use crate::app_state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub mod handler;
pub mod model;
pub mod service;
pub mod store;
#[cfg(test)]
mod tests;

use handler::{
    create_adjustment, get_product_batches, get_product_inventory_records,
    get_product_inventory_stock_level, insert_inventory_record,
};

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route(
            "/{product_id}",
            get(get_product_inventory_records).post(insert_inventory_record),
        )
        .route(
            "/{product_id}/batches",
            get(get_product_batches),
        )
        .route("/low-stock", get(get_product_inventory_stock_level))
        .route("/adjustments", post(create_adjustment))
}
