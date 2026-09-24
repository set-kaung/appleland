use crate::app_error::{APIError, AppError};
use crate::db::DBQueryFilter;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use time::{Date, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct InsertOrder {
    pub sale_id: Uuid,
    pub shipping_address_id: Uuid,
    pub order_method: String,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderRow {
    pub id: Uuid,
    pub sale_id: Uuid,
    // shippinh_
}

#[derive(Debug, Clone, ToSchema, Deserialize)]
pub struct UpdateOrderTrackingURLPayload {
    pub order_tracking_url: String,
}

#[derive(Debug, Clone, ToSchema, Serialize)]
pub struct UpdateOrderDeliveryStatusResponse {
    pub id: Uuid,
    pub delivery_status: DeliveryStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum DeliveryStatus {
    Pending,
    OnDelivery,
    Received,
}

impl DeliveryStatus {
    const fn new() -> Self {
        DeliveryStatus::Pending
    }
    pub fn all() -> Vec<String> {
        let mut values = Vec::new();
        let mut current = Self::new();
        values.push(String::from(&current));
        while let Ok(v) = current.next() {
            values.push(String::from(&v));
            current = v;
        }
        values
    }
    pub fn next(&self) -> Result<Self, APIError> {
        match self {
            DeliveryStatus::Pending => Ok(DeliveryStatus::OnDelivery),
            DeliveryStatus::OnDelivery => Ok(DeliveryStatus::Received),
            DeliveryStatus::Received => Err(APIError::Conflict(
                "already recevied, can't advance".to_string(),
            )),
        }
    }
}

impl TryFrom<&str> for DeliveryStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "on_delivery" => Ok(Self::OnDelivery),
            "received" => Ok(Self::Received),
            _ => Err(AppError::InvalidOperation),
        }
    }
}

impl From<&DeliveryStatus> for String {
    fn from(value: &DeliveryStatus) -> Self {
        match value {
            DeliveryStatus::Pending => "pending".to_string(),
            DeliveryStatus::OnDelivery => "on_delivery".to_string(),
            DeliveryStatus::Received => "received".to_string(),
        }
    }
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct AdminOrderRow {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub customer: Option<String>,
    pub total_amount: Decimal,
    pub delivery_status: String,
    pub order_tracking_url: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct AdminOrdersQuery {
    pub status: Option<String>,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl AdminOrdersQuery {
    pub fn get_limit_and_offset(&self) -> (i64, i64, i64) {
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1).saturating_mul(page_size);

        (page_size, offset, page)
    }
}

impl DBQueryFilter<Postgres> for AdminOrdersQuery {
    fn apply_filter(&self, query_builder: &mut QueryBuilder<Postgres>) {
        if let Some(active_status) = &self.status {
            query_builder.push(" AND o.delivery_status = ");
            query_builder.push_bind(active_status);
        }
        if let Some(new_start_date) = self.start_date {
            query_builder.push(" AND (o.created_at AT TIME ZONE 'Asia/Yangon')::date >= ");
            query_builder.push_bind(new_start_date);
        }
        if let Some(new_end_date) = self.end_date {
            query_builder.push(" AND (o.created_at AT TIME ZONE 'Asia/Yangon')::date <= ");
            query_builder.push_bind(new_end_date);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminOrderDetailRow {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub order_method: String,
    pub delivery_status: String,
    pub order_tracking_url: Option<String>,
    pub customer: Option<String>,
    pub total_amount: Decimal,
    pub is_instore_sale: bool,
    pub payment_method: Option<String>,
    pub payment_provider: Option<String>,
    pub payment_order_id: Option<String>,
    pub shipping_address_line_1: String,
    pub shipping_address_phone_number: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminOrderItemRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub selling_price: Decimal,
    pub cost_price: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminOrderStatusHistoryRow {
    pub id: Uuid,
    pub status: String,
    pub changed_by: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminOrderDetail {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub order_method: String,
    pub delivery_status: String,
    pub order_tracking_url: Option<String>,
    pub customer: Option<String>,
    pub total_amount: Decimal,
    pub is_instore_sale: bool,
    pub payment_method: Option<String>,
    pub payment_provider: Option<String>,
    pub payment_order_id: Option<String>,
    pub shipping_address: AdminOrderShippingAddress,
    pub items: Vec<AdminOrderItem>,
    pub status_history: Vec<AdminOrderStatusEntry>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminOrderShippingAddress {
    pub address_line_1: String,
    pub phone_number: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminOrderStatusEntry {
    pub id: Uuid,
    pub status: String,
    pub changed_by: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminOrderItem {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub selling_price: Decimal,
    pub cost_price: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
