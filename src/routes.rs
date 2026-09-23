use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode},
    middleware as axum_middleware,
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::app_state::AppState;
use crate::{app_error::AppError, orders, staff};
use crate::{
    auth, category, image_storage, inventory, middleware, playground, product, sales, user,
};
use sqlx::Connection;

#[derive(Serialize)]
struct HealthResponse<'a> {
    server: &'a str,
    database: &'a str,
}

async fn health(State(pool): State<PgPool>) -> (StatusCode, Json<HealthResponse<'static>>) {
    let database = match pool.acquire().await {
        Ok(mut conn) => {
            if conn.ping().await.is_ok() {
                "ok"
            } else {
                "error"
            }
        }
        Err(_) => "error",
    };

    (
        StatusCode::OK,
        Json(HealthResponse {
            server: "ok",
            database,
        }),
    )
}

pub fn create_app(
    state: AppState,
    pool: PgPool,
    session_layer: SessionManagerLayer<PostgresStore>,
    allowed_origins: Vec<HeaderValue>,
    burst_size: u32,
    per_sec: u64,
) -> Result<Router, AppError> {
    let root_routes = Router::new().route("/health", get(health)).with_state(pool);

    let account_setup_routes = Router::new().nest("/users", user::route_account_setup());

    let user_protected_routes = Router::new()
        .nest("/products", product::route())
        .nest("/playground", playground::route())
        .nest("/sales", sales::route())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::user_row_middleware,
        ));

    let api_routes = account_setup_routes.merge(user_protected_routes).layer(
        axum_middleware::from_fn_with_state(state.clone(), middleware::user_auth_middleware),
    );

    let admin_protected_routes = Router::new()
        .nest("/staff", staff::route_admin())
        .nest("/products", product::route_admin())
        .nest("/inventory", inventory::route_admin())
        .nest("/categories", category::route_admin())
        .nest("/sales", sales::route_admin())
        .nest("/playground", playground::route_admin())
        .nest("/orders", orders::route_admin())
        .nest("/uploads", image_storage::route())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_auth_middleware,
        ));

    let admin_routes = Router::new()
        .route("/login", axum::routing::post(auth::admin_login_handler))
        .route("/logout", axum::routing::post(auth::admin_logout_handler))
        .merge(admin_protected_routes)
        .layer(session_layer)
        .layer(axum_middleware::from_fn(
            middleware::partitioned_cookie_middleware,
        ))
        .with_state(state.clone());

    let webhook_routes = state.payment_provider.webhook_route();

    let app_router = Router::new()
        .merge(root_routes)
        .merge(webhook_routes)
        .nest("/api", api_routes)
        .nest("/admin", admin_routes)
        .with_state(state);

    middleware::with_global_middleware(app_router, allowed_origins, burst_size, per_sec)
}
