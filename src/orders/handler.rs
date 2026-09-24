use crate::{
    ApiResult, ApiResultWithStatusCode, PaginatedApiResult,
    app_extractor::{RequiresRole, SuperAdminRole},
    app_state::AppState,
    orders::{
        model::{AdminOrderRow, AdminOrdersQuery, UpdateOrderDeliveryStatusResponse, UpdateOrderTrackingURLPayload},
        service, store,
    },
    pagination,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use reqwest::StatusCode;
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

    let (data, total_items) = store::get_all_orders_paginated(&state.db, &query, limit, offset).await?;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
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
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderTrackingURLPayload>,
) -> ApiResultWithStatusCode<Uuid> {
    let result =
        store::update_order_tracking_url(&state.db, order_id, payload.order_tracking_url).await?;
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
    Path(order_id): Path<Uuid>,
) -> ApiResult<UpdateOrderDeliveryStatusResponse> {
    let result = service::update_order_delivery_status(&state.db, order_id).await?;
    Ok(Json(result))
}
