use crate::{
    ApiResult, ApiResultWithStatusCode, PaginatedApiResult,
    app_extractor::{RequiresRole, SuperAdminRole},
    app_state::AppState,
    auth::{CurrentUser, session_admin_id},
    orders::{
        model::{
            AdminOrderDetail, AdminOrderRow, AdminOrdersQuery, UpdateOrderDeliveryStatusResponse,
            UpdateOrderTrackingURLPayload, UserOrderDetail, UserOrderHistoryResponse,
        },
        service, store,
    },
    pagination,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use reqwest::StatusCode;
use tower_sessions::Session;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/admin/orders",
    description = "List paginated orders with optional status and date range filters.",
    params(AdminOrdersQuery),
    responses(
        (status = 200, description = "Paginated list of orders", body = pagination::PaginatedResponse<AdminOrderRow>),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Orders"
)]
pub async fn get_all_orders_paginated(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<AdminOrdersQuery>,
) -> PaginatedApiResult<AdminOrderRow> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (data, total_items) =
        store::get_all_orders_paginated(&state.db, &query, limit, offset).await?;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}

#[utoipa::path(
    get,
    path = "/admin/orders/{order_id}",
    description = "Get full admin order detail including items, customer, shipping address, payment info, and delivery status history.",
    params(
        ("order_id" = Uuid, Path, description = "Order UUID")
    ),
    responses(
        (status = 200, description = "Admin order detail", body = AdminOrderDetail),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Orders"
)]
pub async fn get_admin_order_detail(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> ApiResult<AdminOrderDetail> {
    let result = service::get_admin_order_detail(&state.db, order_id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    patch,
    path = "/admin/orders/{order_id}",
    description = "Update an order tracking url.",
    params(
        ("order_id" = Uuid, Path, description = "Order UUID")
    ),
    request_body = UpdateOrderTrackingURLPayload,
    responses(
        (status = 200, description = "Order tracking url updated", body = Uuid),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Orders"
)]

pub async fn update_order_tracking_url_handler(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderTrackingURLPayload>,
) -> ApiResultWithStatusCode<Uuid> {
    let changed_by = Some(session_admin_id(session).await?);
    let result = service::update_order_tracking_url(
        &state.db,
        order_id,
        payload.order_tracking_url,
        changed_by,
    )
    .await?;
    Ok((StatusCode::OK, Json(result)))
}

#[utoipa::path(
    patch,
    path = "/admin/orders/update_status/{order_id}",
    description = "Update an order delivery status",
    params(("order_id" = Uuid, Path, description= "Order ID/UUID")),
    responses(
        (status = 200, description = "Order delivery status updated", body = UpdateOrderDeliveryStatusResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
     security(
        ("session_cookie" = [])
    ),
    tag = "Admin Orders"
)]
pub async fn update_order_delivery_status(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Path(order_id): Path<Uuid>,
) -> ApiResult<UpdateOrderDeliveryStatusResponse> {
    let changed_by = Some(session_admin_id(session).await?);
    let result = service::update_order_delivery_status(&state.db, order_id, changed_by).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/orders",
    description = "List the current user's past orders (overview) with delivery status, order time and total amount, paginated.",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of the user's orders", body = pagination::PaginatedResponse<UserOrderHistoryResponse>),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "User Orders"
)]
pub async fn get_user_orders_paginated(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<pagination::PaginationQuery>,
) -> PaginatedApiResult<UserOrderHistoryResponse> {
    let user_id = user.id;
    let (limit, offset, current_page) = query.get_limit_and_offset();
    let (rows, total_items) =
        store::fetch_user_orders_paginated(&state.db, user_id, limit, offset).await?;
    let data = rows.into_iter().map(Into::into).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}

#[utoipa::path(
    get,
    path = "/api/orders/{order_id}",
    description = "Get the full detail of one of the current user's orders: items, shipping address, payment info, delivery status, tracking url and status history. Returns 404 if the order does not exist or belongs to another user.",
    params(
        ("order_id" = Uuid, Path, description = "Order UUID")
    ),
    responses(
        (status = 200, description = "Order detail", body = UserOrderDetail),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "User Orders"
)]
pub async fn get_user_order_detail(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
) -> ApiResult<UserOrderDetail> {
    let result = service::get_user_order_detail(&state.db, order_id, user.id).await?;
    Ok(Json(result))
}
