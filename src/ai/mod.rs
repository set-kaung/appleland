use axum::{Router, routing::post};

use crate::ai::handler::chat;
use crate::app_state::AppState;

pub mod handler;
pub mod service;
pub mod store;

pub fn route() -> Router<AppState> {
    Router::new().route("/admin/ai/chat", post(chat))
}