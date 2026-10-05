use crate::analytics::model::{
    ByCategoryFilter, CategoryPerformanceRow, ExpiringSoonQuery, ExpiringSoonRow,
    ProductPerformanceFilter, ProductPerformanceRow,
};
use crate::analytics::store;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::pagination::{self, PaginationQuery};
use crate::{ApiResult, PaginatedApiResult};
use axum::Json;
use axum::extract::{Query, State};

#[utoipa::path(
    get,
    path = "/admin/analytics/ai/expiring-soon",
    description = "SuperAdmin-only expiry report. Returns all inventory batches with quantity_remaining > 0 whose expiry date falls between today and today + days, inclusive, using Asia/Yangon calendar dates. Defaults to 30 days; days=0 returns batches expiring today. Excludes earlier expiry dates and batches without an expiry date. Results are unpaginated and ordered by expiry_date, then batch_id, ascending. expiry_date is returned as an RFC 3339 timestamp.",
    params(ExpiringSoonQuery),
    responses(
        (status = 200, description = "All matching inventory batches", body = Vec<ExpiringSoonRow>),
        (status = 400, description = "Invalid days query parameter; must be an integer from 0 to 65535"),
        (status = 403, description = "Forbidden - Requires a SuperAdmin session"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin AI Analytics"
)]
pub async fn get_expiring_soon(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<ExpiringSoonQuery>,
) -> ApiResult<Vec<ExpiringSoonRow>> {
    let data = store::get_expiring_soon(&state.db, query.days).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/admin/analytics/ai/product-performance",
    description = "Product performance for the AI assistant / manager dashboards. Aggregates sale items over the given Myanmar date range. Defaults to all history if no dates are supplied.",
    params(PaginationQuery, ProductPerformanceFilter),
    responses(
        (status = 200, description = "Paginated product performance", body = pagination::PaginatedResponse<ProductPerformanceRow>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin AI Analytics"
)]
pub async fn get_product_performance(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(filter): Query<ProductPerformanceFilter>,
    Query(pagination): Query<PaginationQuery>,
) -> PaginatedApiResult<ProductPerformanceRow> {
    let (limit, offset, current_page) = pagination.get_limit_and_offset();
    let (data, total_items) = store::get_product_performance(&state.db, &filter, limit, offset).await?;

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
    path = "/admin/analytics/ai/by-category",
    description = "Category-level performance for the AI assistant / manager dashboards. Aggregates sale items by product category over the given Myanmar date range.",
    params(PaginationQuery, ByCategoryFilter),
    responses(
        (status = 200, description = "Paginated category performance", body = pagination::PaginatedResponse<CategoryPerformanceRow>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin AI Analytics"
)]
pub async fn get_category_performance(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(filter): Query<ByCategoryFilter>,
    Query(pagination): Query<PaginationQuery>,
) -> PaginatedApiResult<CategoryPerformanceRow> {
    let (limit, offset, current_page) = pagination.get_limit_and_offset();
    let (data, total_items) = store::get_category_performance(&state.db, &filter, limit, offset).await?;

    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}
