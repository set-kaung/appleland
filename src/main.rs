#![warn(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
use appleland::app_error::AppError;
use appleland::app_state::AppState;
use appleland::auth;
use appleland::config;
use appleland::db;
use appleland::middleware;
use appleland::{api_doc, sales};
use appleland::{category, inventory, product};
use axum::middleware as axum_middleware;
use axum::{Router, http::StatusCode};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use time::Duration;
use tokio::signal;
use tokio::task::AbortHandle;
use tower_sessions::ExpiredDeletion;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("axum_tracing_example=error,tower_http=warn"))
                .map_err(|e| AppError::StartupErr(format!("failed to set up tracing: {e}")))?,
        )
        .init();

    let cli = config::Cli::parse();
    if let Err(e) = api_doc::generate_docs() {
        tracing::error!("failed to generate docs: {e}");
    }
    if cli.gen_docs_only {
        return Ok(());
    }

    let conf = config::load_config()?;
    let session_delete_interval =
        parse_duration::parse(&conf.session_delete_interval).map_err(|e| {
            tracing::error!("failed to parse duration: {}", e);
            AppError::GeneralErr
        })?;

    let pool = db::connect_db(conf.database_url).await?;

    let state = AppState::new(
        Arc::new(auth::SupabaseAuth::new(&conf.supabase_ref)),
        pool.clone(),
    );

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(session_delete_interval),
    );

    let abort_handle = deletion_task.abort_handle();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(tower_sessions::cookie::SameSite::None)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    let root_routes = Router::new().route("/health", axum::routing::get(health));

    let api_routes = Router::new().nest("/products", product::route()).layer(
        axum_middleware::from_fn_with_state(state.clone(), middleware::user_auth_middleware),
    );

    let admin_protected_routes = Router::new()
        .nest("/products", product::route_admin())
        .nest("/inventory", inventory::route_admin())
        .nest("/categories", category::route_admin())
        .nest("/sales", sales::route_admin())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_auth_middleware,
        ));

    let admin_routes = Router::new()
        .route("/login", axum::routing::post(auth::admin_login_handler))
        .route("/logout", axum::routing::post(auth::admin_logout_handler))
        .merge(admin_protected_routes)
        .layer(session_layer)
        .layer(axum::middleware::from_fn(
            middleware::partitioned_cookie_middleware,
        ))
        .with_state(state.clone());

    let app_router = Router::new()
        .merge(root_routes)
        .nest("/api", api_routes)
        .nest("/admin", admin_routes)
        .with_state(state.clone());

    let app: Router = middleware::with_global_middleware(
        app_router,
        conf.allowed_origins,
        conf.burst_size,
        conf.per_sec,
    )?;

    tracing::info!("Starting api server on port :{}", conf.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", conf.port))
        .await
        .map_err(|e| AppError::StartupErr(format!("failed to listen: {e}")))?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(abort_handle.clone()))
    .await
    .map_err(|_| AppError::GeneralErr)?;

    match deletion_task.await {
        Ok(_) => tracing::info!("Session deletion task completed normally"),
        Err(e) if e.is_cancelled() => {
            tracing::info!("Session deletion task cancelled via shutdown");
        }
        Err(e) => tracing::error!("Session deletion task failed: {}", e),
    }
    Ok(())
}

async fn health() -> Result<(StatusCode, String), ()> {
    Ok((StatusCode::OK, "service is up".to_string()))
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!("failed to install Ctrl+C handler: {e}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("failed to install signal handler: {e}");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { deletion_task_abort_handle.abort() },
        () = terminate => { deletion_task_abort_handle.abort() },
    }
}
