pub mod handler;
pub mod model;
mod store;

use axum::routing::post;
use axum::{Router, routing::get};
use handler::*;

use crate::app_state::AppState;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products_paginated))
        .route("/{product_id}", get(get_product_by_id))
        .route("/search", get(find_products))
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
        .route("/search", get(find_products_admin))
        .route("/search/advanced", post(find_products_admin_advanced))
}
