use crate::app_state::AppState;
use crate::category::handler::{create_category, get_all_categories, update_category};
use axum::Router;
use axum::routing::get;

pub mod handler;
pub mod model;
pub mod store;

pub fn route_admin() -> Router<AppState> {
    axum::Router::new()
        .route(
            "/",
            get(get_all_categories).post(create_category),
        )
        .route("/{id}", axum::routing::put(update_category))
}
