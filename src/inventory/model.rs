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
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub received_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option", default)]
    pub expiry_date: Option<OffsetDateTime>,
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
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub received_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option", default)]
    pub expiry_date: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewInventoryPayload {
    pub quantity_received: i32,
    pub unit_cost: Decimal,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub received_at: Option<OffsetDateTime>,

    #[serde(with = "time::serde::rfc3339::option", default)]
    pub expiry_date: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewInventoryRecord {
    pub product_id: Uuid,
    pub quantity_received: i32,
    pub quantity_remaining: i32,
    pub unit_cost: Decimal,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub received_at: Option<OffsetDateTime>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub expiry_date: Option<OffsetDateTime>,
}

impl NewInventoryRecord {
    pub fn new(
        product_id: Uuid,
        admin_id: Uuid,
        product_is_perishable: bool,
        payload: &NewInventoryPayload,
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

        if product_is_perishable && payload.expiry_date.is_none() {
            return Err(APIError::BadRequest(
                "expiry_date is required for perishable products".to_string(),
            ));
        }

        if !product_is_perishable && payload.expiry_date.is_some() {
            return Err(APIError::BadRequest(
                "expiry_date cannot be set for non-perishable products".to_string(),
            ));
        }

        Ok(NewInventoryRecord {
            product_id,
            quantity_received: payload.quantity_received,
            quantity_remaining: payload.quantity_received,
            unit_cost: payload.unit_cost,
            created_by: admin_id,
            updated_by: admin_id,
            received_at: payload.received_at,
            expiry_date: payload.expiry_date,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DeductInventoryPayload {
    pub quantity_to_deduct: i32,
}

#[derive(Debug, FromRow)]
pub struct StockLevelRow {
    pub product_name: String,
    pub product_id: Uuid,
    pub quantity_in_stock: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StockLevel {
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity_in_stock: i32,
    pub is_low_stock: bool,
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
            received_at: value.received_at,

            expiry_date: value.expiry_date,
        }
    }
}
