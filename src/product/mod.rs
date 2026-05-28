mod error;
pub mod handler;
pub mod model;
mod store;

use crate::app_state::AppState;
use axum::{Router, routing::get};
use handler::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products).post(handler::create_product))
        .route(
            "/{product_id}",
            get(get_product_by_id)
                .patch(handler::update_product)
                .delete(handler::delete_product),
        )
}
