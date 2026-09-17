use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserPayload {
    pub address_line_1: String,
    pub phone_number: String,
}
