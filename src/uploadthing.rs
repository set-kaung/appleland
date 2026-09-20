use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ApiResult;
use crate::app_error::APIError;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct PresignRequest {
    pub filename: String,
    pub content_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct PresignResponse {
    pub upload_url: String,
    pub file_url: String,
}

#[utoipa::path(
    post,
    path = "/admin/uploads/presign",
    description = "Get a presigned UploadThing URL for file uploads.",
    request_body = PresignRequest,
    responses(
        (status = 200, description = "Presigned upload URL and public file URL", body = PresignResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Uploads"
)]
pub async fn get_upload_presigned_url(
    State(state): State<AppState>,
    _role: RequiresRole<SuperAdminRole>,
    Json(payload): Json<PresignRequest>,
) -> ApiResult<PresignResponse> {
    let client = reqwest::Client::new();

    let res = client
        .post("https://api.uploadthing.com/v6/prepareUpload")
        .header("x-uploadthing-api-key", &state.uploadthing_token)
        .json(&serde_json::json!({
            "files": [{
                "name": payload.filename,
                "size": 1048576,
                "type": payload.content_type
            }]
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("failed to call uploadthing API: {e}");
            APIError::InternalErr
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::error!(%status, %body, "uploadthing API returned error");
        return Err(APIError::InternalErr);
    }

    let body: serde_json::Value = res.json().await.map_err(|e| {
        tracing::error!("failed to parse uploadthing response: {e}");
        APIError::InternalErr
    })?;

    let file_data = body
        .get("data")
        .and_then(|d| d.get(0))
        .ok_or_else(|| {
            tracing::error!("missing data array in uploadthing response");
            APIError::InternalErr
        })?;

    let upload_url = file_data
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::error!("missing upload url in uploadthing response");
            APIError::InternalErr
        })?
        .to_string();

    let file_url = file_data
        .get("fileUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::error!("missing file url in uploadthing response");
            APIError::InternalErr
        })?
        .to_string();

    Ok(Json(PresignResponse {
        upload_url,
        file_url,
    }))
}

pub fn route() -> axum::Router<AppState> {
    axum::Router::new().route("/presign", axum::routing::post(get_upload_presigned_url))
}
