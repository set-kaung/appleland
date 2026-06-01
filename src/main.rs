use appleland::app_error::AppError;
use appleland::app_state::AppState;
use appleland::auth;
use appleland::config;
use appleland::db;
use appleland::middleware;
use appleland::product;
use axum::middleware as axum_middleware;
use axum::{Router, http::StatusCode};

use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();

    let conf = config::load_config()?;
    let pool = db::connect_db(conf.database_url).await?;

    let app_state = AppState::new(Arc::new(auth::SupabaseAuth::new(conf.supabase_ref)), pool);

    let root_routes = Router::new().route("/health", axum::routing::get(health));
    let api_routes = Router::new().nest("/products", product::router());

    let app_router = Router::new()
        .merge(root_routes)
        .nest(
            &format!("/api/v{}", conf.api_version),
            // apply auth_middleware to all routes in api_routes
            api_routes.route_layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                middleware::auth_middleware,
            )),
        )
        .with_state(app_state);

    let app = middleware::with_global_middleware(
        app_router,
        conf.allowed_origins,
        conf.burst_size,
        conf.per_sec,
    );

    println!(
        "Starting api server (v{}) on port :{}",
        conf.api_version, conf.port
    );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", conf.port))
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}

async fn health() -> Result<(StatusCode, String), ()> {
    Ok((StatusCode::OK, "service is up".to_string()))
}
