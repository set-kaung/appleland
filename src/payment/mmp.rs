use axum::http::HeaderMap;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::{PaymentError, PaymentInfo, PaymentProvider};
use crate::app_error::APIError;
use crate::app_state::AppState;

type HmacSha256 = Hmac<Sha256>;

const SANDBOX_BASE_URL: &str = "https://sandbox.mmpay.com";
const PRODUCTION_BASE_URL: &str = "https://api.mmpay.com";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MmpayStatus {
    Pending,
    Success,
    Failed,
    Refunded,
    Expired,
    Cancelled,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MmpayCondition {
    Prestine,
    Touched,
    Expired,
    Dirty,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MmpayMethod {
    Qr,
    Pin,
    Pwa,
    Card,
}

#[derive(Debug, Deserialize)]
pub struct MmpayWebhookBody {
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
    pub vendor: String,
    pub method: MmpayMethod,
    pub status: MmpayStatus,
    pub condition: MmpayCondition,
    pub transaction_ref_id: String,
    pub vendor_qr_ref_id: String,
    #[serde(default)]
    pub callback_url: Option<String>,
    #[serde(default)]
    pub custom_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MmpayWebhookResponse {
    pub status: String,
}

// -- Handshake response --

#[derive(Debug, Deserialize)]
struct MmpayHandshakeResponse {
    token: String,
}

// -- Get payment info response --

#[derive(Debug, Deserialize)]
pub struct MmpayPaymentInfoResponse {
    pub order_id: String,
    #[serde(default)]
    pub app_id: Option<String>,
    pub amount: f64,
    pub vendor: String,
    pub method: String,
    #[serde(default)]
    pub custom_message: Option<String>,
    #[serde(default)]
    pub callback_url: Option<String>,
    #[serde(default)]
    pub callback_url_at: Option<String>,
    #[serde(default)]
    pub callback_url_status: Option<String>,
    pub status: String,
    #[serde(default)]
    pub disbursement_id: Option<String>,
    #[serde(default)]
    pub dis_status: Option<String>,
    pub condition: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub transaction_ref_id: Option<String>,
    #[serde(default)]
    pub vendor_qr_ref_id: Option<String>,
    #[serde(default)]
    pub qr: Option<String>,
}

// -- Provider --

pub struct MyanMyanPay {
    pub webhook_secret: String,
    pub publishable_key: String,
    pub sandbox: bool,
}

impl MyanMyanPay {
    pub fn new(webhook_secret: String, publishable_key: String, sandbox: bool) -> Self {
        Self {
            webhook_secret,
            publishable_key,
            sandbox,
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox {
            SANDBOX_BASE_URL
        } else {
            PRODUCTION_BASE_URL
        }
    }

    fn sign_request(&self, body: &[u8], nonce: &str) -> Result<String, PaymentError> {
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| PaymentError::Serialize(format!("failed to create HMAC: {e}")))?;
        mac.update(body);
        mac.update(nonce.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    fn auth_headers(
        &self,
        body_bytes: &[u8],
        nonce: &str,
    ) -> Result<Vec<(&str, String)>, PaymentError> {
        let signature = self.sign_request(body_bytes, nonce)?;
        Ok(vec![
            ("Content-Type", "application/json".to_string()),
            ("Authorization", format!("Bearer {}", self.publishable_key)),
            ("X-Mmpay-Nonce", nonce.to_string()),
            ("X-Mmpay-Signature", signature),
        ])
    }

    fn status_is_paid(status: &str) -> bool {
        matches!(status, "SUCCESS")
    }
}

#[async_trait::async_trait]
impl PaymentProvider for MyanMyanPay {
    async fn handshake(&self, payment_order_id: &str) -> Result<String, PaymentError> {
        let payload = serde_json::json!({ "orderId": payment_order_id });
        let body_bytes =
            serde_json::to_vec(&payload).map_err(|e| PaymentError::Serialize(e.to_string()))?;

        let nonce = uuid::Uuid::new_v4().to_string();
        let headers = self.auth_headers(&body_bytes, &nonce)?;

        let handshake_path = if self.sandbox {
            "sandbox-handshake"
        } else {
            "handshake"
        };
        let url = format!("{}/payments/{}", self.base_url(), handshake_path);

        let client = reqwest::Client::new();
        let mut req = client.post(&url);
        for (key, value) in &headers {
            req = req.header(*key, value);
        }

        let resp = req
            .body(body_bytes)
            .send()
            .await
            .map_err(|e| PaymentError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(PaymentError::Provider(format!(
                "handshake HTTP {status}: {text}"
            )));
        }

        let raw: MmpayHandshakeResponse = resp
            .json()
            .await
            .map_err(|e| PaymentError::InvalidResponse(e.to_string()))?;

        Ok(raw.token)
    }

    async fn get_payment_info(
        &self,
        payment_order_id: &str,
        btoken: &str,
    ) -> Result<PaymentInfo, PaymentError> {
        let payload = serde_json::json!({ "orderId": payment_order_id });
        let body_bytes =
            serde_json::to_vec(&payload).map_err(|e| PaymentError::Serialize(e.to_string()))?;

        let nonce = uuid::Uuid::new_v4().to_string();
        let headers = self.auth_headers(&body_bytes, &nonce)?;

        let get_path = if self.sandbox { "sandbox-get" } else { "get" };
        let url = format!("{}/payments/{}", self.base_url(), get_path);

        let client = reqwest::Client::new();
        let mut req = client.post(&url);
        for (key, value) in &headers {
            req = req.header(*key, value);
        }
        req = req.header("X-Mmpay-Btoken", btoken);

        let resp = req
            .body(body_bytes)
            .send()
            .await
            .map_err(|e| PaymentError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(PaymentError::Provider(format!(
                "get_payment_info HTTP {status}: {text}"
            )));
        }

        let raw: MmpayPaymentInfoResponse = resp
            .json()
            .await
            .map_err(|e| PaymentError::InvalidResponse(e.to_string()))?;

        Ok(PaymentInfo {
            payment_order_id: raw.order_id,
            amount: raw.amount,
            currency: "MMK".to_string(),
            vendor: raw.vendor,
            method: raw.method,
            status: raw.status,
            condition: raw.condition,
            transaction_ref_id: raw.transaction_ref_id,
            vendor_qr_ref_id: raw.vendor_qr_ref_id,
            created_at: raw.created_at,
        })
    }

    async fn is_paid(&self, payment_order_id: &str) -> Result<Option<PaymentInfo>, PaymentError> {
        let btoken = self.handshake(payment_order_id).await?;
        let info = self.get_payment_info(payment_order_id, &btoken).await?;
        if Self::status_is_paid(&info.status) {
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    fn provider_name(&self) -> &'static str {
        "mmpay"
    }

    fn generate_signature(&self, body: &[u8], nonce: &str) -> Result<String, PaymentError> {
        self.sign_request(body, nonce)
    }

    fn verify_signature(&self, body: &[u8], signature: &str, nonce: &str) -> bool {
        let Ok(mut mac) = HmacSha256::new_from_slice(self.webhook_secret.as_bytes()) else {
            return false;
        };
        mac.update(body);
        mac.update(nonce.as_bytes());

        let Ok(expected_bytes) = hex::decode(signature) else {
            return false;
        };

        mac.verify_slice(&expected_bytes).is_ok()
    }

    fn webhook_route(&self) -> Router<AppState> {
        Router::new().route("/payment", post(handle_mmpay_webhook))
    }
}

// -- Webhook handler --

async fn handle_mmpay_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    raw_body: axum::body::Bytes,
) -> Result<(StatusCode, Json<MmpayWebhookResponse>), APIError> {
    let signature = headers
        .get("X-Mmpay-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(APIError::BadRequest("missing X-Mmpay-Signature".into()))?;

    let nonce = headers
        .get("X-Mmpay-Nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or(APIError::BadRequest("missing X-Mmpay-Nonce".into()))?;

    if !state
        .payment_provider
        .verify_signature(&raw_body, signature, nonce)
    {
        return Err(APIError::Unauthorized);
    }

    if !super::store::consume_nonce(&state.db, nonce).await? {
        tracing::info!(
            nonce,
            "duplicate webhook nonce, acking to stop provider retries"
        );
        return Ok((
            StatusCode::OK,
            Json(MmpayWebhookResponse {
                status: "received".to_string(),
            }),
        ));
    }

    let body: MmpayWebhookBody = serde_json::from_slice(&raw_body)
        .map_err(|e| APIError::BadRequest(format!("invalid webhook body: {e}")))?;

    tracing::info!(
        order_id = %body.order_id,
        status = ?body.status,
        amount = body.amount,
        currency = %body.currency,
        "mmpay webhook received"
    );

    match body.status {
        MmpayStatus::Success => {
            tracing::info!(
                order_id = %body.order_id,
                transaction_ref_id = %body.transaction_ref_id,
                "payment successful"
            );
        }
        MmpayStatus::Failed | MmpayStatus::Cancelled => {
            tracing::warn!(
                order_id = %body.order_id,
                status = ?body.status,
                "payment failed or cancelled"
            );
        }
        MmpayStatus::Expired => {
            tracing::warn!(order_id = %body.order_id, "payment expired");
        }
        _ => {}
    }

    Ok((
        StatusCode::OK,
        Json(MmpayWebhookResponse {
            status: "received".to_string(),
        }),
    ))
}
