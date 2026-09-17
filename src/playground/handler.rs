use axum::extract::Path;
use axum::{Extension, Json, extract::State};
use tower_sessions::Session;
use uuid::Uuid;

use crate::app_extractor::ValidatedJson;
use crate::playground::model::{
    ClaimTokenPayload, UserPlaygroundAccount, UserPlaygroundPurchaseResponse,
};
use crate::playground::service;
use crate::{
    ApiResult,
    app_error::APIError,
    app_extractor::{PlaygroundAdminRole, RequiresRole},
    app_state::AppState,
    auth::{CurrentUser, password::session_admin_id},
    playground::{
        model::{CreateClaimTokenPayload, UserPlaygroundTickets},
        store,
    },
};

#[utoipa::path(
    post,
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
    if payload.total_quantity <= 0 {
        return Err(APIError::BadRequest(
            "ticket quantity must be greater than 0".to_string(),
        ));
    }
    let admin_id = session_admin_id(session).await?;
    let token = store::create_claim_token(
        &state.db,
        admin_id,
        payload.total_quantity,
        payload.unit_price,
    )
    .await?;
    Ok(Json(token))
}

#[utoipa::path(get,
     path="/api/playground/tickets",
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
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<UserPlaygroundTickets> {
    let user_id = user.id;
    let result = match store::get_user_tickets_info(&state.db, user_id).await? {
        Some(row) => row.into(),
        None => UserPlaygroundTickets {
            user_id,
            tickets_purchased: 0,
            free_tickets_redeemed: 0,
            free_ticket_eligible: 0,
        },
    };
    Ok(Json(result))
}

#[utoipa::path(patch,
    path="/api/playground/tokens/{token_id}",
    description = "Claim a ticket sale token by a user",
    request_body(
        content = ClaimTokenPayload,
        description = "Optional list of playground coupon IDs to redeem as free tickets. If omitted, no free tickets are redeemed.",
    ),
    responses(
        (status = 200, description = "Claimed Response", body = UserPlaygroundPurchaseResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 410, description = "Token Expired"),
        (status = 500, description= "Internal server error")
    ),
    security(("bearer" = [])),
    tag = "User Playground")]
pub async fn claim_token(
    State(state): State<AppState>,
    Path(token_id): Path<Uuid>,
    Extension(user): Extension<CurrentUser>,
    ValidatedJson(payload): ValidatedJson<ClaimTokenPayload>,
) -> ApiResult<UserPlaygroundPurchaseResponse> {
    let user_id = user.id;
    let result =
        service::claim_token(&state.db, user_id, token_id, payload.playground_coupons).await?;
    Ok(Json(result))
}

#[utoipa::path(patch,
    path="/api/playground/tickets/claim_free",
    description = "Claim a free ticket by a user",
    responses(
        (status = 200, description = "Claimed Response", body = UserPlaygroundAccount),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 500, description= "Internal server error")
    ),
    security(("bearer" = [])),
    tag = "User Playground")]
pub async fn claim_free_ticket(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<UserPlaygroundAccount> {
    let user_id = user.id;
    let result = service::claim_free_ticket(&state.db, user_id).await?;
    Ok(Json(result))
}
