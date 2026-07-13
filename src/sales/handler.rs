use super::store;
use crate::app_error::APIError;
use crate::app_state::AppState;
use crate::pagination;
use crate::sales::model::{AdminSale, AdminSaleDetails};
use axum::Json;
use axum::extract::{Path, Query, State};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/sales",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of sale records", body = pagination::PaginatedResponse<AdminSale>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Sales"
)]
pub async fn get_sales_paginated(
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
    path = "/api/products/{sale_id}",
    params(
        ("sale_id" = Uuid, Path, description = "Sale UUID")
    ),
    responses(
        (status = 200, body = AdminSaleDetails),
        (status = 404, description = "Sale Detail not found"),
        (status = 500)
    ),
    tag = "Admin Sales"
)]
pub async fn get_sale_detail_by_id(
    State(state): State<AppState>,
    Path(sale_id): Path<Uuid>,
) -> Result<Json<AdminSaleDetails>, APIError> {
    let data = store::fetch_sale_detail_by_id(&state.db, sale_id).await?;
    Ok(Json(data))
}
