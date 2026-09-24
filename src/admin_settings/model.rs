use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

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

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "string" => Some(SettingValueType::String),
            "integer" => Some(SettingValueType::Integer),
            "decimal" => Some(SettingValueType::Decimal),
            "boolean" => Some(SettingValueType::Boolean),
            _ => None,
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SettingRow {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub r#type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}
#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct SettingResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub r#type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SettingPayload {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub r#type: SettingValueType,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct NewSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub r#type: SettingValueType,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl From<SettingRow> for SettingResponse {
    fn from(sr: SettingRow) -> Self {
        Self {
            key: sr.key,
            value: sr.value,
            description: sr.description,
            r#type: sr.r#type,
            created_at: sr.created_at,
            updated_at: sr.updated_at,
            created_by: sr.created_by,
            updated_by: sr.updated_by,
        }
    }
}
