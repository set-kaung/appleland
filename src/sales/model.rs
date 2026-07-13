use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize)]
pub struct SaleRow {
    pub id: Uuid,
    pub total_amount: Decimal,
    pub created_at: OffsetDateTime,
}

#[derive(FromRow, Serialize, Deserialize, ToSchema)]
pub struct AdminSale {
    pub id: Uuid,
    pub total_amount: Decimal,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct AdminSaleDetails {
    pub id: Uuid,
    pub total_amount: Decimal,
    pub created_at: OffsetDateTime,
    pub sale_items: Vec<SaleItem>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct SaleItem {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sale_id: Uuid,
    pub quantity: i32,
    pub selling_price: Decimal,
    pub cost_price: Decimal,
    pub created_at: OffsetDateTime,
}

impl From<SaleRow> for AdminSale {
    fn from(value: SaleRow) -> Self {
        Self {
            id: value.id,
            total_amount: value.total_amount,
            created_at: value.created_at,
        }
    }
}
