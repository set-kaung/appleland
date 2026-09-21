pub mod mmp;
pub mod store;

use std::time::Duration;

use axum::Router;
use serde::Serialize;
use sqlx::PgPool;

use crate::app_state::AppState;

const NONCE_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Serialize)]
pub struct PaymentInfo {
    pub payment_order_id: String,
    pub amount: f64,
    pub currency: String,
    pub vendor: String,
    pub method: String,
    pub status: String,
    pub condition: String,
    pub transaction_ref_id: Option<String>,
    pub vendor_qr_ref_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PaymentError {
    #[error("provider error: {0}")]
    Provider(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync + 'static {
    async fn handshake(&self, payment_order_id: &str) -> Result<String, PaymentError>;
    async fn get_payment_info(
        &self,
        payment_order_id: &str,
        btoken: &str,
    ) -> Result<PaymentInfo, PaymentError>;
    async fn is_paid(&self, payment_order_id: &str) -> Result<Option<PaymentInfo>, PaymentError>;
    fn provider_name(&self) -> &'static str;
    fn generate_signature(&self, body: &[u8], nonce: &str) -> Result<String, PaymentError>;
    fn verify_signature(&self, body: &[u8], signature: &str, nonce: &str) -> bool;
    fn webhook_route(&self) -> Router<AppState>;
}

pub fn spawn_nonce_cleanup(pool: PgPool, interval: Duration) -> tokio::task::AbortHandle {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            match store::delete_expired_nonces(&pool, NONCE_RETENTION).await {
                Ok(deleted) if deleted > 0 => {
                    tracing::debug!(deleted, "webhook nonce sweep");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = ?e, "webhook nonce sweep failed");
                }
            }
        }
    })
    .abort_handle()
}
