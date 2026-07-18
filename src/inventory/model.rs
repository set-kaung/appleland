use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct InventoryRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity_received: i32,
    pub quantity_remaining: i32,
    pub unit_cost: Decimal,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewInventoryRecord {
    pub product_id: Uuid,
    pub quantity_received: i32,
    pub quantity_remaining: i32,
    pub unit_cost: Decimal,
    pub created_by: Uuid,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AddInventoryRecord {
    pub product_id: Uuid,
    pub quantity_remaining: i32,
    pub quantity_received: i32,
    pub unit_cost: Decimal,
    pub update_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DeductInventoryPayload {
    pub quantity_remaining_sub: i32,
}
