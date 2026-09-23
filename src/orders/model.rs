use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_error::AppError;

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
        loop {
            match current.next() {
                Ok(v) => {
                    values.push(String::from(&v));
                    current = v;
                }
                Err(_) => break,
            }
        }
        values
    }
    pub fn next(&self) -> Result<Self, AppError> {
        match self {
            DeliveryStatus::Pending => Ok(DeliveryStatus::OnDelivery),
            DeliveryStatus::OnDelivery => Ok(DeliveryStatus::Received),
            DeliveryStatus::Received => Err(AppError::InvalidOperation),
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
