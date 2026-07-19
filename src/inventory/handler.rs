use crate::app_state::AppState;
use crate::auth::password::session_admin_id;
use crate::inventory::model::{AdminInventory, NewInventoryPayload, NewInventoryRecord};
use crate::ApiResult;
use axum::extract::{Path, State};
use axum::Json;
use tower_sessions::Session;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/admin/inventory/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = NewInventoryPayload,
    responses(
        (status = 200, body = AdminInventory),
        (status = 400),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Inventory"
)]
pub async fn insert_inventory(
    State(state): State<AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<NewInventoryPayload>,
) -> ApiResult<AdminInventory> {
    let admin_id: Uuid = session_admin_id(session).await?;

    let inventory_record = NewInventoryRecord {
        product_id,
        quantity_received: payload.quantity_received,
        quantity_remaining: payload.quantity_received,
        unit_cost: payload.unit_cost,
        created_by: admin_id,
        updated_by: admin_id,
    };
    let result = super::store::insert_new_inventory_record(inventory_record, &state.db).await?.into();
    Ok(Json(result))
}
