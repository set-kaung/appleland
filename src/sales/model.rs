use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use time::{Date, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::db::DBQueryFilter;

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
pub struct SaleProduct {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct CreateSalePayload {
    pub sale_products: Vec<SaleProduct>,
}

#[derive(Debug, Clone)]
pub struct SaleProductDTO {
    pub product_id: Uuid,
    pub quantity: i32,
    pub cost_price: Decimal,
    pub inventory_id: Uuid,
    pub selling_price: Decimal,
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

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SaleSummaryFilter {
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
}

impl DBQueryFilter<Postgres> for SaleSummaryFilter {
    fn apply_filter(&self, query_builder: &mut sqlx::QueryBuilder<Postgres>) {
        if let Some(start_date) = self.start_date {
            query_builder.push(" AND s.created_at::date >= ");
            query_builder.push_bind(start_date);
        }
        if let Some(end_date) = self.end_date {
            query_builder.push(" AND s.created_at::date <= ");
            query_builder.push_bind(end_date);
        }
    }
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct SaleSummaryRow {
    pub sale_date: Date,
    pub total_sale: Decimal,
    pub is_online_sale: bool,
    pub transactions: i32,
    pub items_sold: i32,
    pub total_cost: Decimal,
    pub margin: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SaleSummary {
    pub sale_date: Date,
    pub total_sale: Decimal,
    pub is_online_sale: bool,
    pub transactions: i32,
    pub items_sold: i32,
    pub total_cost: Decimal,
    pub margin: Decimal,
    pub margin_percentage: f64,
    pub avg_basket: Decimal,
}

impl From<SaleSummaryRow> for SaleSummary {
    fn from(value: SaleSummaryRow) -> Self {
        let avg_basket = if value.transactions == 0 {
            Decimal::ZERO
        } else {
            value.total_sale / Decimal::from(value.transactions)
        };
        let margin_percentage = if value.total_sale == Decimal::ZERO {
            0.0
        } else {
            ((value.margin / value.total_sale) * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        };
        Self {
            sale_date: value.sale_date,
            total_sale: value.total_sale,
            is_online_sale: value.is_online_sale,
            transactions: value.transactions,
            items_sold: value.items_sold,
            total_cost: value.total_cost,
            margin: value.margin,
            margin_percentage,
            avg_basket,
        }
    }
}
