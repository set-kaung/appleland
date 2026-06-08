pub mod handler;
pub mod model;
mod store;

use crate::app_state::{AdminState, ApiState};
use axum::{Router, routing::get};
use handler::*;

pub fn route_api_states() -> Router<ApiState> {
    Router::new()
        .route("/", get(list_products::<ApiState>))
        .route("/{product_id}", get(get_product_by_id::<ApiState>))
}

pub fn route_admin_states() -> Router<AdminState> {
    Router::new()
        .route("/", get(list_products::<AdminState>).post(create_product))
        .route(
            "/{product_id}",
            get(get_product_by_id::<AdminState>)
                .delete(delete_product)
                .put(update_product),
        )
}
