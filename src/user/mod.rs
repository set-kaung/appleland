pub mod handler;
pub mod model;
pub mod service;
mod store;

use axum::{Router, routing::post};

use crate::app_state::AppState;

pub fn route() -> Router<AppState> {
    Router::new().route("/setup", post(handler::create_user))
}
