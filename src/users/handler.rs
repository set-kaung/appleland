use crate::app_error::APIError;
use crate::app_extractor::{RequiresRole, SuperAdminRole, ValidatedJson};
use crate::app_state::AppState;
use crate::auth::Claims;
use crate::users::model::{CreateUserPayload, UserResponse};
use crate::users::{service, store};
use crate::{ApiResult, PaginatedApiResult, pagination};
use axum::extract::{Query, State};
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

#[utoipa::path(get, path = "/admin/users")]
pub async fn get_all_users_paginated(
    State(state): State<AppState>,
    _require_role: RequiresRole<SuperAdminRole>,
    Query(query): Query<pagination::PaginationQuery>,
) -> PaginatedApiResult<UserResponse> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (data, total_items) = store::get_all_users_paginated(&state.db, limit, offset).await?;

    let data = data.into_iter().map(Into::into).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}
