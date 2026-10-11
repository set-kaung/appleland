pub mod handlers;
pub mod model;
pub mod service;
mod store;

use axum::Router;
use axum::routing::{get, patch};

use crate::app_state::AppState;

pub fn route_admin() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_all_settings))
        .route("/{key}", patch(handlers::update_setting_value))
}
