use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Inventory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UpdateInventory {
    pub product_id: Uuid,
    pub quantity: i32,
    pub update_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UpdateInventoryPayload {
    pub quantity: i32,
}
