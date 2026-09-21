pub mod mmp;

use axum::Router;
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct PaymentInfo {
    pub order_id: String,
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
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync + 'static {
    async fn handshake(&self, order_id: &str) -> Result<String, PaymentError>;
    async fn get_payment_info(
        &self,
        order_id: &str,
        btoken: &str,
    ) -> Result<PaymentInfo, PaymentError>;
    fn is_paid(&self, info: &PaymentInfo) -> bool;
    fn generate_signature(&self, body: &[u8], nonce: &str) -> String;
    fn verify_signature(&self, body: &[u8], signature: &str, nonce: &str) -> bool;
    fn webhook_route(&self) -> Router<AppState>;
}
