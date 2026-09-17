use crate::ApiResult;
use crate::app_error::APIError;
use crate::app_extractor::ValidatedJson;
use crate::app_state::AppState;
use crate::auth::Claims;
use crate::user::model::CreateUserPayload;
use crate::user::service;
use axum::extract::State;
use axum::{Extension, Json};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/users/setup",
    description = "Complete account setup for a newly signed-up user with address and phone number.",
    request_body = CreateUserPayload,
    responses(
        (status = 200, description = "Account setup completed successfully, returns user id", body = Uuid),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 400, description = "Bad request - Invalid input"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "User"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateUserPayload>,
) -> ApiResult<Uuid> {
    let supabase_id = Uuid::parse_str(&claims.sub).map_err(|_| APIError::Unauthorized)?;
    let user_id = service::new_user_account_creation(&state.db, supabase_id, payload).await?;
    Ok(Json(user_id))
}
