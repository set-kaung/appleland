use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserPayload {
    pub username: String,
    pub address_line_1: String,
    pub phone_number: String,
}

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub address_line_1: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct UserContactRow {
    pub address_line_1: String,
    pub phone_number: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub address_line_1: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct UserPointsBalanceUpdateResult {
    pub id: Uuid,
    pub points_added: i32,
    pub final_balance: i32,
}

impl From<UserRow> for UserResponse {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            username: value.username,
            address_line_1: value.address_line_1,
            phone_number: value.phone_number,
        }
    }
}
