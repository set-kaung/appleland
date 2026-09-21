#![warn(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use appleland::api_doc;
use appleland::app_error::AppError;
use appleland::app_state::AppState;
use appleland::auth;
use appleland::config;
use appleland::db;
use appleland::payment::mmp::MyanMyanPay;
use appleland::playground;
use appleland::routes;
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
    let json_logs =
        std::env::var("LOG_JSON").is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("appleland=info,tower_http=warn"))
        .map_err(|e| AppError::StartupErr(format!("failed to set up tracing: {e}")))?;

    if json_logs {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

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
    let token_cleanup_interval = parse_duration::parse(&conf.playground_token_cleanup_interval)
        .map_err(|e| {
            tracing::error!("failed to parse duration: {}", e);
            AppError::GeneralErr
        })?;

    let pool = db::connect_db(conf.database_url).await?;

    let state = AppState::new(
        Arc::new(auth::SupabaseAuth::new(&conf.supabase_ref)),
        pool.clone(),
        conf.uploadthing_token,
        Arc::new(MyanMyanPay::new(
            conf.mmpay_webhook_secret,
            conf.mmpay_publishable_key,
            conf.mmpay_sandbox,
        )),
    );

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(session_delete_interval),
    );

    let abort_handle = deletion_task.abort_handle();

    let token_cleanup_handle =
        playground::spawn_expired_token_cleanup(pool.clone(), token_cleanup_interval);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(tower_sessions::cookie::SameSite::None)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    let app = routes::create_app(
        state.clone(),
        pool,
        session_layer,
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
    .with_graceful_shutdown(shutdown_signal(abort_handle.clone(), token_cleanup_handle))
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

async fn shutdown_signal(
    deletion_task_abort_handle: AbortHandle,
    token_cleanup_abort_handle: AbortHandle,
) {
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
        () = ctrl_c => {
            deletion_task_abort_handle.abort();
            token_cleanup_abort_handle.abort();
        }
        () = terminate => {
            deletion_task_abort_handle.abort();
            token_cleanup_abort_handle.abort();
        },
    }
}
