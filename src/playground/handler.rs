use axum::extract::{Path, Query};
use axum::{Extension, Json, extract::State};
use tower_sessions::Session;
use uuid::Uuid;

use crate::app_extractor::ValidatedJson;
use crate::playground::model::{
    AdminPlaygroundPurchase, AdminPlaygroundPurchasesQuery, ClaimTokenPayload,
    PlaygroundCheckoutData, PlaygroundCheckoutDataAdmin, TicketSaleSummary,
    TicketSaleSummaryFilter, UserPlaygroundAccount, UserPlaygroundPurchaseResponse,
};
use crate::playground::service;
use crate::{
    ApiResult, PaginatedApiResult,
    app_error::APIError,
    app_extractor::{PlaygroundAdminRole, RequiresRole, SuperAdminRole},
    app_state::AppState,
    auth::{CurrentUser, session_admin_id},
    pagination,
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
    let result = match store::get_user_playground_account(&state.db, user_id).await? {
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

#[utoipa::path(
    patch,
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

///api/playground/checkout/5a36d0c5-291e-4e56-96c8-ad270c433e77
#[utoipa::path(
    get,
    path="/api/playground/checkout/{token_id}",
    description = "Checkout page data for a playground token",
    responses(
        (status = 200, description = "Claimed Response", body = PlaygroundCheckoutData),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 500, description= "Internal server error")
    ),
    security(("bearer" = [])),
    tag = "User Playground")]
pub async fn get_checkout_data(
    State(state): State<AppState>,
    Path(token_id): Path<Uuid>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<PlaygroundCheckoutData> {
    let result =
        service::get_playground_checkout_data_service(&state.db, Some(user.id), token_id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/admin/playground/checkout/{token_id}",
    description = "Checkout data for a playground claim token from the admin side. The claiming user (claimed_by) is read from the token itself; coupons are applied for that user. Unclaimed tokens return `claimed_by` = null.",
    params(
        ("token_id" = Uuid, Path, description = "Claim token id"),
    ),
    responses(
        (status = 200, description = "Claimed Response", body = PlaygroundCheckoutDataAdmin),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin Playground")]
pub async fn get_checkout_data_admin(
    RequiresRole(_role, _): RequiresRole<PlaygroundAdminRole>,
    State(state): State<AppState>,
    Path(token_id): Path<Uuid>,
) -> ApiResult<PlaygroundCheckoutDataAdmin> {
    let result = service::get_playground_checkout_data_admin_service(&state.db, token_id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/admin/playground/summary",
    description = "Daily playground ticket sales summary (paid tickets only) between the given dates, inclusive.",
    params(TicketSaleSummaryFilter),
    responses(
        (status = 200, description = "Daily ticket sales summary", body = Vec<TicketSaleSummary>),
        (status = 400, description = "Bad request - invalid or inverted date range"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Playground"
)]
pub async fn get_ticket_sale_summary(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(filter): Query<TicketSaleSummaryFilter>,
) -> ApiResult<Vec<TicketSaleSummary>> {
    if filter.start_date > filter.end_date {
        return Err(APIError::BadRequest(
            "start_date must be on or before end_date".to_string(),
        ));
    }
    let rows =
        store::get_ticket_sale_summary(&state.db, filter.start_date, filter.end_date).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/admin/playground/purchases",
    description = "Paginated list of playground ticket purchases, newest first. Optional filters: date range on Asia/Yangon calendar days (inclusive) and paid vs free redemption. `line_total` is quantity x unit_price for paid purchases and 0 for free redemptions.",
    params(AdminPlaygroundPurchasesQuery),
    responses(
        (status = 200, description = "Paginated list of playground purchases", body = pagination::PaginatedResponse<AdminPlaygroundPurchase>),
        (status = 400, description = "Bad request - invalid or inverted date range"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Playground"
)]
pub async fn get_admin_purchases_paginated(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<AdminPlaygroundPurchasesQuery>,
) -> PaginatedApiResult<AdminPlaygroundPurchase> {
    let inverted_range = query
        .start_date
        .zip(query.end_date)
        .is_some_and(|(start_date, end_date)| start_date > end_date);
    if inverted_range {
        return Err(APIError::BadRequest(
            "start_date must be on or before end_date".to_string(),
        ));
    }
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (data, total_items) =
        service::get_admin_purchases_paginated(&state.db, &query, limit, offset).await?;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}
