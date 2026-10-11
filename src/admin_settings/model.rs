use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueType {
    String,
    Integer,
    Decimal,
    Boolean,
}

impl SettingValueType {
    pub fn as_str(self) -> &'static str {
        match self {
            SettingValueType::String => "string",
            SettingValueType::Integer => "integer",
            SettingValueType::Decimal => "decimal",
            SettingValueType::Boolean => "boolean",
        }
    }

    pub fn is_valid(self, value: &serde_json::Value) -> bool {
        match self {
            SettingValueType::String => value.is_string(),
            SettingValueType::Integer => value.as_i64().is_some(),
            SettingValueType::Decimal => value.is_number(),
            SettingValueType::Boolean => value.is_boolean(),
        }
    }
}

impl TryFrom<&str> for SettingValueType {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "string" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "decimal" => Ok(Self::Decimal),
            "boolean" => Ok(Self::Boolean),
            _ => Err(AppError::InvalidOperation),
        }
    }
}

impl std::fmt::Display for SettingValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SettingRow {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub value_type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Option<Uuid>,
}
#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct SettingResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub value_type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Option<Uuid>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SettingPayload {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub value_type: SettingValueType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSettingPayload {
    pub value: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct NewSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub value_type: SettingValueType,
    pub updated_by: Option<Uuid>,
}

impl From<SettingRow> for SettingResponse {
    fn from(sr: SettingRow) -> Self {
        Self {
            key: sr.key,
            value: sr.value,
            description: sr.description,
            value_type: sr.value_type,
            created_at: sr.created_at,
            updated_at: sr.updated_at,
            updated_by: sr.updated_by,
        }
    }
}
