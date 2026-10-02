use axum::body::Bytes;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use std::time::Duration;
use uuid::Uuid;

use crate::ai::store;
use crate::app_error::APIError;
use crate::app_state::AppState;

const GEMINI_API_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(60);

pub struct RelayResponse {
    pub status: StatusCode,
    pub content_type: HeaderValue,
    pub body: Bytes,
}

pub async fn chat_service(
    state: &AppState,
    admin_id: Uuid,
    body: Bytes,
) -> Result<RelayResponse, APIError> {
    let used = store::get_today_count(&state.db, admin_id).await?;
    if i64::from(used) >= i64::from(state.ai.daily_limit) {
        return Err(APIError::TooManyRequests(
            "daily ai chat limit exceeded".to_string(),
        ));
    }

    let client = reqwest::Client::new();
    let url = format!("{}/{}:generateContent", GEMINI_API_BASE, state.ai.gemini_model);
    let upstream = client
        .post(url)
        .header("x-goog-api-key", &state.ai.gemini_api_key)
        .header(CONTENT_TYPE, "application/json")
        .timeout(UPSTREAM_TIMEOUT)
        .body(body)
        .send()
        .await;

    let response = match upstream {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(error = ?e, "gemini upstream request failed");
            record_usage(state, admin_id).await;
            return Err(APIError::GatewayTimeout);
        }
    };

    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("application/json"));
    let body = response.bytes().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to read gemini upstream response body");
        APIError::InternalErr
    })?;

    if status.is_success() || status == StatusCode::GATEWAY_TIMEOUT {
        record_usage(state, admin_id).await;
    }

    Ok(RelayResponse {
        status,
        content_type,
        body,
    })
}

async fn record_usage(state: &AppState, admin_id: Uuid) {
    if let Err(e) = store::increment_today_count(&state.db, admin_id).await {
        tracing::error!(error = ?e, "failed to record ai chat usage");
    }
}