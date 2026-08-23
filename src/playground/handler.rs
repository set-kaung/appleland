use axum::{Extension, Json, extract::{Path, State}};
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
        (status = 401, description = "Unauthorized - Missing or invalid Session Cookie"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
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
    if payload.quantity < 0{
        return Err(APIError::BadRequest("ticket quantity must be greater than 0".to_string()))
    }
    let admin_id = session_admin_id(session).await?;
    let token = store::create_claim_token(&state.db, admin_id, payload.quantity).await?;
    Ok(Json(token))
}

#[utoipa::path(get,
     path="/api/playground/tickets/{user_id}",
     description = "Get Tickets History of a User",
     responses(
        (status = 200, description = "Ticket History", body = UserPlaygroundTickets),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 500, description= "Internal server error")
     ),
     security(
        ("bearer" = [])
    ),
    tag = "User Playground"
    )]
pub async fn get_user_tickets(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<UserPlaygroundTickets> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| APIError::Unauthorized)?;
    let result = store::get_user_tickets_info(&state.db, user_id)
        .await?
        .into();
    Ok(Json(result))
}

#[utoipa::path(patch,
    path="/api/playground/tokens/{token_id}",
    description = "Claim a ticket sale token by a user",
    responses(
        (status = 200, description = "Ticket Token Claimed", body = Uuid),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 410, description = "Token Expired"),
        (status = 500, description= "Internal server error")
    ),
    security(("bearer" = [])),
    tag = "User Playground")]
pub async fn claim_token(State(state): State<AppState>, Path(token_id): Path<Uuid>,Extension(claims): Extension<Claims>,) -> ApiResult<Uuid>{
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| APIError::Unauthorized)?;
    let result = store::claim_token(&state.db, user_id, token_id).await?;
    Ok(Json(result))
}