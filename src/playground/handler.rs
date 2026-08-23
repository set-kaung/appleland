use axum::{Extension, Json, extract::State};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    ApiResult,
    app_error::APIError,
    app_extractor::{PlaygroundAdminRole, RequiresRole},
    app_state::AppState,
    auth::{Claims, password::session_admin_id},
    playground::{
        model::{CreateClaimTokenPayload, UserPlaygroundTickets},
        store,
    },
};

#[utoipa::path(
    get,
    path = "/admin/playground/tokens",
    description = "Create a claim token for the frontend",
    request_body = CreateClaimTokenPayload,
    responses(
        (status = 200, description = "Claim token", body = Uuid),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Admin Playground"
)]
#[axum::debug_handler]
pub async fn creat_playground_token(
    RequiresRole(_role, _): RequiresRole<PlaygroundAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<CreateClaimTokenPayload>,
) -> ApiResult<Uuid> {
    let admin_id = session_admin_id(session).await?;
    let token = store::create_claim_token(&state.db, admin_id, payload.quantity).await?;
    Ok(Json(token))
}

pub async fn get_user_ticket(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<UserPlaygroundTickets> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| APIError::Unauthorized)?;
    let result = store::get_user_tickets_info(&state.db, user_id)
        .await?
        .into();
    Ok(Json(result))
}
