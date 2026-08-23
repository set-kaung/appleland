use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use std::time::Duration;

use crate::{
    app_state::AppState,
    playground::handler::{creat_playground_token, get_user_ticket},
};

pub mod handler;
pub mod model;
mod store;

pub fn spawn_expired_token_cleanup(pool: PgPool, interval: Duration) -> tokio::task::AbortHandle {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            match store::delete_expired_tokens(&pool).await {
                Ok(n) if n > 0 => {
                    tracing::debug!(deleted = n, "cleaned up expired playground tokens");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = ?e, "playground token cleanup failed");
                }
            }
        }
    })
    .abort_handle()
}

pub fn route() -> Router<AppState> {
    Router::new().route("/tickets", get(get_user_ticket))
}

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/tokens", post(creat_playground_token))
}
