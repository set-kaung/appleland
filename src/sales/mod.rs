use axum::Router;
use axum::routing::{get};
use crate::app_state::AppState;
use crate::sales::handler::{get_sale_detail_by_id, get_sales_paginated};

pub mod model;
pub mod store;
pub mod handler;





pub fn route_admin() -> Router<AppState>{
    Router::new()
        .route("/", get(get_sales_paginated))
        .route("/{sale_id}", get(get_sale_detail_by_id))
}