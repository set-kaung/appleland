use super::store;
use crate::app_error::{APIError, StoreError};
use crate::app_extractor::{RequiresRole, SaleAdminRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::auth::CurrentUser;
use crate::auth::session_admin_id;
use crate::sales::model::{
    AdminSale, AdminSaleDetails, CreateAdminSalePayload, CreateUserSalePayload, SaleSummary,
    SaleSummaryFilter,
};
use crate::sales::service;
use crate::{ApiResult, ApiResultWithStatusCode, pagination};
use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tower_sessions::Session;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/admin/sales",
    description = "List all sales with pagination.",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of sale records", body = pagination::PaginatedResponse<AdminSale>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Sales"
)]
pub async fn get_sales_paginated(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<AdminSale>>, APIError> {
    let (limit, offset, current_page) = query.get_limit_and_offset();
    let (result, total_items) = store::fetch_all_paginated(&state.db, limit, offset).await?;

    let data = result.into_iter().map(|x| x.into()).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    // AdminSale implement From<SaleRow>
    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}

#[utoipa::path(
    get,
    path = "/admin/sales/{sale_id}",
    description = "Get full details of a single sale including its line items.",
    params(
        ("sale_id" = Uuid, Path, description = "Sale UUID")
    ),
    responses(
        (status = 200, description = "Sale details", body = AdminSaleDetails),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 404, description = "Sale not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Sales"
)]
pub async fn get_sale_detail_by_id(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    Path(sale_id): Path<Uuid>,
) -> Result<Json<AdminSaleDetails>, APIError> {
    let data = service::get_admin_sale_detail_by_id(&state.db, sale_id).await?;
    Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/admin/sales",
    description = "Create a new sale (POS checkout) with FIFO inventory deduction.",
    request_body = CreateAdminSalePayload,
    responses(
        (status = 201, description = "Sale created", body = Uuid),
        (status = 400, description = "Invalid request or insufficient inventory"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Sales"
)]
pub async fn create_admin_sale(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<CreateAdminSalePayload>,
) -> ApiResultWithStatusCode<Uuid> {
    let admin_id = session_admin_id(session).await?;
    let aggregated_products = service::validate_aggregate_sale_products(&payload.sale_products)?;
    let sale_id = service::perform_sale_admin(&state.db, admin_id, &aggregated_products).await?;
    Ok((StatusCode::CREATED, Json(sale_id)))
}

#[utoipa::path(
    get,
    path = "/admin/sales/summary",
    description = "Aggregated daily sales summary with margin and basket metrics.",
    params(SaleSummaryFilter),
    responses(
        (status = 200, description = "Daily sales summary", body = Vec<SaleSummary>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Sales"
)]
pub async fn sale_summary(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(filter): Query<SaleSummaryFilter>,
) -> ApiResult<Vec<SaleSummary>> {
    let rows = store::get_sale_summary(&state.db, filter).await?;
    Ok(Json(rows.into_iter().map(|row| row.into()).collect()))
}

#[utoipa::path(
    post,
    path = "/api/sales",
    description = "Create a new sale (checkout) with FIFO inventory deduction.",
    request_body = CreateUserSalePayload,
    responses(
        (status = 201, description = "Sale created", body = Uuid),
        (status = 400, description = "Invalid request or insufficient inventory"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 402, description = "Payment not confirmed for order"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "User Sales"
)]
pub async fn create_user_sale(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(payload): Json<CreateUserSalePayload>,
) -> ApiResultWithStatusCode<Uuid> {
    let aggregated_products = service::validate_aggregate_sale_products(&payload.sale_products)?;
    let Some(payment_info) = state
        .payment_provider
        .is_paid(&payload.payment_order_id)
        .await?
    else {
        return Err(APIError::PaymentNotConfirmed);
    };

    let expected_total = service::compute_sale_total(&state.db, &aggregated_products).await?;
    let paid_amount = Decimal::from_f64(payment_info.amount)
        .ok_or_else(|| APIError::BadRequest("invalid payment amount".into()))?;
    if paid_amount != expected_total {
        return Err(APIError::BadRequest(
            "payment amount does not match sale total".into(),
        ));
    }

    match service::perform_sale_user(
        &state.db,
        user.id,
        &aggregated_products,
        &payload.payment_order_id,
        state.payment_provider.provider_name(),
        &payment_info.method,
    )
    .await
    {
        Ok(sale_id) => Ok((StatusCode::CREATED, Json(sale_id))),
        Err(StoreError::UniqueKeyViolated(_)) => {
            let sale_id = store::fetch_sale_id_by_payment_order_id(
                &state.db,
                &payload.payment_order_id,
                user.id,
            )
            .await?;
            Ok((StatusCode::CREATED, Json(sale_id)))
        }
        Err(e) => Err(e.into()),
    }
}
