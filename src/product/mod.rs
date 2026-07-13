pub mod handler;
pub mod model;
mod store;

use axum::{Router, routing::get};
use axum::routing::post;
use handler::*;

use crate::app_state::AppState;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products_paginated))
        .route("/{product_id}", get(get_product_by_id))
}

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products_paginated_admin).post(create_product))
        .route("/all", get(get_all_products))
        .route(
            "/{product_id}",
            get(get_product_by_id_admin)
                .delete(delete_product)
                .put(update_product),
        )
        .route("/search", post(find_products_admin))
}
