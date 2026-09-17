use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub address_line_1: String,
    pub phone_number: String,
}
