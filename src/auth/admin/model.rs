use std::str::FromStr;

use crate::orders::model::DeliveryStatus;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    ToSchema,
)]
#[sqlx(type_name = "admin_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    SaleAdmin,
    PlaygroundAdmin,
    SuperAdmin,
}

impl FromStr for AdminRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sale_admin" => Ok(AdminRole::SaleAdmin),
            "playground_admin" => Ok(AdminRole::PlaygroundAdmin),
            "super_admin" => Ok(AdminRole::SuperAdmin),
            _ => Err("not defined role".to_string()),
        }
    }
}

impl std::fmt::Display for AdminRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self {
            AdminRole::SaleAdmin => "sale_admin",
            AdminRole::PlaygroundAdmin => "playground_admin",
            AdminRole::SuperAdmin => "super_admin",
        };
        write!(f, "{}", str_rep)
    }
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Clone)]
#[allow(dead_code)]
pub struct AdminRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: AdminRole,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Admin {
    pub id: Uuid,
    pub username: String,
    pub role: AdminRole,
    pub delivery_status_values: Vec<String>,
}

impl From<AdminRow> for Admin {
    fn from(value: AdminRow) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role,
            delivery_status_values: DeliveryStatus::all(),
        }
    }
}
