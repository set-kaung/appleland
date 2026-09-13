use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct CreateClaimTokenPayload {
    pub total_quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct UserPlaygroundTickets {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub to_redeem: i32,
}

#[derive(Debug, FromRow)]
pub struct PlaygroundTicketsRow {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserPlaygroundPurchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: i32,
    pub is_free: bool,
}

#[derive(FromRow)]
pub struct ExpiredStatus {
    pub expired: bool,
    pub status: String,
}

#[derive(FromRow)]
pub struct ClaimedTokenInfo {
    pub id: Uuid,
    pub claimed_by: Uuid,
    pub total_quantity: i32,
    pub unit_price: Decimal,
    pub created_by: Uuid,
}

#[derive(FromRow)]
pub struct PlaygroundPurchaseRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: i32,
    pub is_free_redemption: bool,
}

#[derive(FromRow)]
pub struct CouponStatus {
    pub id: Uuid,
    pub is_not_used: bool,
    pub is_expired: bool,
}

impl From<PlaygroundTicketsRow> for UserPlaygroundTickets {
    fn from(value: PlaygroundTicketsRow) -> Self {
        let to_redeem = (value.tickets_purchased / 10).saturating_sub(value.free_tickets_redeemed);
        Self {
            user_id: value.user_id,
            tickets_purchased: value.tickets_purchased,
            free_tickets_redeemed: value.free_tickets_redeemed,
            to_redeem,
        }
    }
}
