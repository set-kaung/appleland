use crate::app_state::AppState;
use crate::sales::handler::{create_sale, get_sale_detail_by_id, get_sales_paginated};
use axum::Router;
use axum::routing::get;

pub mod handler;
pub mod model;
pub mod service;
pub mod store;

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/", get(get_sales_paginated).post(create_sale))
        .route("/{sale_id}", get(get_sale_detail_by_id))
}
