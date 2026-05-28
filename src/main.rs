mod app_state;
mod auth;
mod core;
mod db;
mod middleware;
mod product;

use app_state::AppState;
use axum::middleware as axum_middleware;
use axum::{Router, error_handling::HandleErrorLayer, http::StatusCode};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tower::{BoxError, ServiceBuilder, timeout::TimeoutLayer};
use tracing::{error, info};

use crate::middleware::auth_middleware::auth_middleware;

#[derive(Deserialize)]
struct Config {
    port: u16,
    supabase_ref: String,
    api_version: u8,
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let port;
    let supabase_ref: String;
    let api_ver: u8;
    let conn_str: String;
    match envy::from_env::<Config>() {
        Ok(config) => {
            port = config.port;
            supabase_ref = config.supabase_ref;
            api_ver = config.api_version;
            conn_str = config.database_url;
        }
        Err(err) => {
            eprintln!("fatal err: failed to load config. err message: {}", err);
            std::process::exit(1);
        }
    }
    tracing_subscriber::fmt::init();

    let pool = db::connect_db(conn_str).await?;
    let options = pool.connect_options();
    info!(
        "Connected to: {}:{}",
        options.get_host(),
        options.get_port()
    );
    let app_state = AppState::new(Arc::new(auth::SupabaseAuth::new(supabase_ref)), pool);

    let api_routes = Router::new().nest("/products", product::router());

    let timeout_middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            if err.is::<tower::timeout::error::Elapsed>() {
                (
                    StatusCode::GATEWAY_TIMEOUT,
                    "The request took too long to process".to_string(),
                )
            } else {
                error!("internal server error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("internal error"))
            }
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let app = Router::new()
        .nest(
            &format!("/api/v{}/", api_ver),
            api_routes.route_layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .with_state(app_state)
        .layer(timeout_middleware);

    println!("Starting api server (v{}) on port :{}", api_ver, port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
