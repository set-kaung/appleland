use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
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
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
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
pub struct InsertProduct {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductPayload {
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
    pub updated_by: Uuid,
}
