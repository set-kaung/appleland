use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AdminProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub category: String,
    pub sub_category_id: Option<Uuid>,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
    pub quantity_in_stock: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub price: Decimal,
    pub quantity_in_stock: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductPayload {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct InsertProduct {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
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
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub price: Decimal,
    pub updated_by: Uuid,
}

#[derive(Debug, FromRow)]
pub struct ProductRow {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub category: String,
    pub sub_category_id: Option<Uuid>,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub price: Decimal,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub quantity_in_stock: i32,
}

impl From<ProductRow> for UserProduct {
    fn from(row: ProductRow) -> Self {
        Self {
            id: row.id,
            barcode: row.barcode,
            name: row.name,
            description: row.description,
            category: row.category,
            sub_category: row.sub_category,
            price: row.price,
            quantity_in_stock: row.quantity_in_stock,
        }
    }
}

impl From<ProductRow> for AdminProduct {
    fn from(row: ProductRow) -> Self {
        Self {
            id: row.id,
            barcode: row.barcode,
            name: row.name,
            description: row.description,
            category_id: row.category_id,
            category: row.category,
            sub_category_id: row.sub_category_id,
            sub_category: row.sub_category,
            is_active: row.is_active,
            price: row.price,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
            quantity_in_stock: row.quantity_in_stock,
        }
    }
}
