use axum::{
    Router,
    routing::{get, patch, post},
};
use sqlx::PgPool;
use std::time::Duration;

use crate::playground::handler::claim_free_ticket;
use crate::{
    app_state::AppState,
    playground::handler::{claim_token, creat_playground_token, get_user_tickets,get_checkout_data},
};

pub mod handler;
pub mod model;
pub mod service;
mod store;

pub fn spawn_expired_token_cleanup(pool: PgPool, interval: Duration) -> tokio::task::AbortHandle {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            match store::sweep_expired_tokens(&pool).await {
                Ok((deleted, marked)) if deleted > 0 || marked > 0 => {
                    tracing::debug!(deleted, marked, "playground token sweep");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = ?e, "playground token sweep failed");
                }
            }
        }
    })
    .abort_handle()
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/tickets", get(get_user_tickets))
        .route("/tokens/{token_id}", patch(claim_token))
        .route("/tickets/claim_free", patch(claim_free_ticket))
        .route("/playground/checkout/{token_id}", get(get_checkout_data))
}

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/tokens", post(creat_playground_token))
}
