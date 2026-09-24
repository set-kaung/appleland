pub mod handler;
pub mod model;
pub mod service;
pub mod store;

use axum::{
    Router,
    routing::{get, post},
};

use crate::app_state::AppState;

pub fn route_account_setup() -> Router<AppState> {
    Router::new().route("/setup", post(handler::create_user))
}

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/", get(handler::get_all_users_paginated))
}
