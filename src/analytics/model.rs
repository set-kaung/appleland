use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{Date, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::db::DBQueryFilter;
use sqlx::Postgres;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExpiringSoonQuery {
    /// Days ahead from today in Asia/Yangon, inclusive. Zero means today only.
    #[serde(default = "default_expiring_soon_days")]
    #[param(default = 30, minimum = 0, maximum = 65535)]
    pub days: u16,
}

fn default_expiring_soon_days() -> u16 {
    30
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct ExpiringSoonRow {
    pub product_id: Uuid,
    pub product_name: String,
    /// Inventory record ID identifying the batch.
    pub batch_id: Uuid,
    pub quantity_remaining: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub expiry_date: OffsetDateTime,
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct ProductPerformanceRow {
    pub product_id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub units_sold: i64,
    pub revenue: Decimal,
    pub cost: Decimal,
    pub profit: Decimal,
    pub margin_pct: Decimal,
    pub share_pct: Option<Decimal>,
    #[serde(with = "time::serde::rfc3339")]
    pub last_sold_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ProductPerformanceFilter {
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub sort: Option<String>,
}

impl DBQueryFilter<Postgres> for ProductPerformanceFilter {
    fn apply_filter(&self, query_builder: &mut sqlx::QueryBuilder<Postgres>) {
        if let Some(start_date) = self.start_date {
            query_builder.push(" AND (s.created_at AT TIME ZONE 'Asia/Yangon')::date >= ");
            query_builder.push_bind(start_date);
        }
        if let Some(end_date) = self.end_date {
            query_builder.push(" AND (s.created_at AT TIME ZONE 'Asia/Yangon')::date <= ");
            query_builder.push_bind(end_date);
        }
    }

    fn apply_sort(&self, query_builder: &mut sqlx::QueryBuilder<Postgres>) {
        let order = match self.sort.as_deref() {
            Some("units") => "units_sold DESC",
            Some("revenue") => "revenue DESC",
            Some("profit") => "profit DESC",
            Some("margin") => "margin_pct DESC",
            _ => "units_sold DESC",
        };
        query_builder.push(format!(" ORDER BY {}", order));
    }
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct CategoryPerformanceRow {
    pub category_id: Option<Uuid>,
    pub category: Option<String>,
    pub units_sold: i64,
    pub revenue: Decimal,
    pub cost: Decimal,
    pub profit: Decimal,
    pub margin_pct: Decimal,
    pub share_pct: Option<Decimal>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ByCategoryFilter {
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub sort: Option<String>,
}

impl DBQueryFilter<Postgres> for ByCategoryFilter {
    fn apply_filter(&self, query_builder: &mut sqlx::QueryBuilder<Postgres>) {
        if let Some(start_date) = self.start_date {
            query_builder.push(" AND (s.created_at AT TIME ZONE 'Asia/Yangon')::date >= ");
            query_builder.push_bind(start_date);
        }
        if let Some(end_date) = self.end_date {
            query_builder.push(" AND (s.created_at AT TIME ZONE 'Asia/Yangon')::date <= ");
            query_builder.push_bind(end_date);
        }
    }

    fn apply_sort(&self, query_builder: &mut sqlx::QueryBuilder<Postgres>) {
        let order = match self.sort.as_deref() {
            Some("units") => "units_sold DESC",
            Some("revenue") => "revenue DESC",
            Some("profit") => "profit DESC",
            Some("margin") => "margin_pct DESC",
            _ => "units_sold DESC",
        };
        query_builder.push(format!(" ORDER BY {}", order));
    }
}
