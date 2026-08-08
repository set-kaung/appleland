use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_error::APIError;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AdminInventory {
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

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
pub struct NewInventoryPayload {
    pub quantity_received: i32,
    pub unit_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewInventoryRecord {
    pub product_id: Uuid,
    pub quantity_received: i32,
    pub quantity_remaining: i32,
    pub unit_cost: Decimal,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl NewInventoryRecord {
    pub fn new(
        product_id: Uuid,
        admin_id: Uuid,
        payload: NewInventoryPayload,
    ) -> Result<Self, APIError> {
        if payload.quantity_received <= 0 {
            return Err(APIError::BadRequest(
                "quantity received must be greater than 0".to_string(),
            ));
        }

        if payload.unit_cost <= 0.into() {
            return Err(APIError::BadRequest(
                "unit cost must be greater than 0".to_string(),
            ));
        }

        Ok(NewInventoryRecord {
            product_id,
            quantity_received: payload.quantity_received,
            quantity_remaining: payload.quantity_received,
            unit_cost: payload.unit_cost,
            created_by: admin_id,
            updated_by: admin_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DeductInventoryPayload {
    pub quantity_to_deduct: i32,
}

impl From<InventoryRow> for AdminInventory {
    fn from(value: InventoryRow) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            quantity_received: value.quantity_received,
            quantity_remaining: value.quantity_remaining,
            unit_cost: value.unit_cost,
            created_by: value.created_by,
            updated_by: value.updated_by,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
