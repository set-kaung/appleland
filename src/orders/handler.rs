use crate::{
    ApiResult, ApiResultWithStatusCode,
    app_extractor::{RequiresRole, SuperAdminRole},
    app_state::AppState,
    orders::{
        model::{UpdateOrderDeliveryStatusResponse, UpdateOrderTrackingURLPayload},
        service, store,
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;
use uuid::Uuid;

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
