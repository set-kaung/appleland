use axum::Json;
use axum::extract::{Multipart, State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::ApiResult;
use crate::app_error::APIError;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
    pub file_url: String,
}

#[utoipa::path(
    post,
    path = "/admin/uploads",
    description = "Upload an image to Supabase Storage. Send as multipart/form-data with a 'file' field.",
    request_body(content = String, description = "Multipart form data with 'file' field", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Public file URL", body = UploadResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Uploads"
)]
pub async fn upload_image(
    State(state): State<AppState>,
    _role: RequiresRole<SuperAdminRole>,
    mut multipart: Multipart,
) -> ApiResult<UploadResponse> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| {
            tracing::error!("failed to read multipart field: {e}");
            APIError::InternalErr
        })?
        .ok_or_else(|| {
            tracing::error!("missing file field in multipart");
            APIError::InternalErr
        })?;

    let original_filename = field
        .file_name()
        .ok_or_else(|| {
            tracing::error!("missing filename in multipart");
            APIError::InternalErr
        })?
        .to_string();

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    let data = field.bytes().await.map_err(|e| {
        tracing::error!("failed to read multipart bytes: {e}");
        APIError::InternalErr
    })?;

    let filename = match original_filename.rsplit_once('.') {
        Some((name, ext)) => format!("{}_{}.{}", name, uuid::Uuid::new_v4(), ext),
        None => format!("{}_{}", original_filename, uuid::Uuid::new_v4()),
    };

    tracing::info!(
        original_filename,
        filename,
        content_type,
        size = data.len(),
        "uploading image to supabase storage"
    );

    let bucket = &state.image_bucket_name;
    let upload_url = format!(
        "https://{}.supabase.co/storage/v1/object/{}/{}",
        state.supabase_ref, bucket, filename
    );

    let client = reqwest::Client::new();
    let res = client
        .post(&upload_url)
        .header("apikey", &state.supabase_secret_key)
        .header(
            "Authorization",
            format!("Bearer {}", state.supabase_secret_key),
        )
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(data)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("failed to upload to supabase storage: {e}");
            APIError::InternalErr
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::error!(%status, %body, "supabase storage upload failed");
        return Err(APIError::InternalErr);
    }

    let file_url = format!(
        "https://{}.supabase.co/storage/v1/object/public/{}/{}",
        state.supabase_ref, bucket, filename
    );

    tracing::debug!(filename, file_url, "image uploaded successfully");

    Ok(Json(UploadResponse { file_url }))
}

pub fn route() -> axum::Router<AppState> {
    axum::Router::new().route("/", axum::routing::post(upload_image))
}
