use axum::{
    Router,
    routing::{get, patch},
};

pub mod handler;
pub mod model;
pub mod service;
mod store;

use handler::*;

use crate::app_state::AppState;

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_orders_paginated))
        .route("/{order_id}", patch(update_order_tracking_url_handler))
        .route(
            "/update_status/{order_id}",
            patch(update_order_delivery_status),
        )
}
