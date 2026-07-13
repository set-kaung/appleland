use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    inventory::{
        model::{Inventory, UpdateInventory, UpdateInventoryPayload},
        store,
    },
};

#[utoipa::path(
    put,
    path = "/admin/inventory/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = UpdateInventoryPayload,
    responses(
        (status = 200, body = Inventory),
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
pub async fn update_inventory(
    State(state): State<AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateInventoryPayload>,
) -> Result<(StatusCode, Json<Inventory>), (StatusCode, String)> {
    let admin_id: Uuid = session
        .get::<Uuid>("id")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "no user_id in session".into()))?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "something went wrong".to_string(),
        )
    })?;

    let update_inventory_data = UpdateInventory {
        product_id,
        quantity: payload.quantity,
        update_by: admin_id,
    };
    match store::update(update_inventory_data, &mut tx).await {
        Ok(result) => {
            tx.commit().await.map_err(|e| {
                error!("failed to commit to database in update_product: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_string(),
                )
            })?;

            Ok((StatusCode::OK, Json(result)))
        }
        Err(e) => {
            error!("failed to update inventory: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ))
        }
    }
}
