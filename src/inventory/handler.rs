use super::store;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::auth::session_admin_id;
use crate::inventory::model::{AdminInventory, NewInventoryPayload, NewInventoryRecord};
use crate::product::service as product_service;
use crate::{ApiResult, PaginatedApiResult, pagination};
use axum::Json;
use axum::extract::{Path, Query, State};
use tower_sessions::Session;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/admin/inventory/{product_id}",
    description = "Add inventory stock to an existing product.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = NewInventoryPayload,
    responses(
        (status = 200, description = "Inventory record created", body = AdminInventory),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Inventory"
)]
pub async fn insert_inventory_record(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<NewInventoryPayload>,
) -> ApiResult<AdminInventory> {
    let admin_id: Uuid = session_admin_id(session).await?;

    let is_perishable = product_service::is_product_perishable(&state.db, product_id).await?;
    let inventory_record = NewInventoryRecord::new(product_id, admin_id, is_perishable, &payload)?;
    let result = store::insert_new_inventory_record(&state.db, inventory_record)
        .await?
        .into();
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/admin/inventory/{product_id}",
    description = "List paginated inventory records for a product.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID"),
        pagination::PaginationQuery
    ),
    responses(
        (status = 200, description = "Paginated inventory records", body = pagination::PaginatedResponse<AdminInventory>),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Inventory"
)]
pub async fn get_product_inventory_records(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
    Path(product_id): Path<Uuid>,
) -> PaginatedApiResult<AdminInventory> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (rows, total_items) =
        store::get_product_inventory_records_paginated(&state.db, product_id, limit, offset)
            .await?;
    let data = rows.into_iter().map(Into::into).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}
