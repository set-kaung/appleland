use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductPayload {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductPayload {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
}
