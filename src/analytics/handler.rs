use crate::analytics::model::{ProductPerformanceFilter, ProductPerformanceRow};
use crate::analytics::store;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::pagination::{self, PaginationQuery};
use crate::PaginatedApiResult;
use axum::Json;
use axum::extract::{Query, State};

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
