use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::ops::Mul;
use time::{Date, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct CreateClaimTokenPayload {
    pub total_quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct TokenInfo {
    pub total_quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct UserPlaygroundTickets {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub free_ticket_eligible: i32,
}

#[derive(Debug, FromRow)]
pub struct PlaygroundTicketsRow {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub free_ticket_eligible: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, FromRow)]
pub struct PlaygroundAccountRow {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub free_ticket_eligible: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPlaygroundAccount {
    pub user_id: Uuid,
    pub tickets_purchased: i32,
    pub free_tickets_redeemed: i32,
    pub free_ticket_eligible: i32,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct UserPlaygroundPurchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: i32,
    pub is_free_redemption: bool,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct UserPlaygroundPurchaseResponse {
    pub purchased_record: Option<UserPlaygroundPurchase>,
    pub free_record: Option<UserPlaygroundPurchase>,
}

#[derive(FromRow)]
pub struct ExpiredStatus {
    pub expired: bool,
    pub status: String,
}

#[derive(Clone, Debug, ToSchema, Deserialize)]
pub struct ClaimTokenPayload {
    pub playground_coupons: Option<Vec<Uuid>>,
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
    pub created_at: OffsetDateTime,
    pub created_by: Uuid,
    pub unit_price: Decimal,
}

#[derive(FromRow)]
pub struct CouponStatus {
    pub id: Uuid,
    pub is_used: bool,
    pub is_expired: bool,
}

impl From<PlaygroundTicketsRow> for UserPlaygroundTickets {
    fn from(value: PlaygroundTicketsRow) -> Self {
        Self {
            user_id: value.user_id,
            tickets_purchased: value.tickets_purchased,
            free_tickets_redeemed: value.free_tickets_redeemed,
            free_ticket_eligible: value.free_ticket_eligible,
        }
    }
}

impl From<PlaygroundPurchaseRow> for UserPlaygroundPurchase {
    fn from(value: PlaygroundPurchaseRow) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            quantity: value.quantity,
            is_free_redemption: value.is_free_redemption,
        }
    }
}

impl From<PlaygroundAccountRow> for UserPlaygroundAccount {
    fn from(value: PlaygroundAccountRow) -> Self {
        Self {
            user_id: value.user_id,
            tickets_purchased: value.tickets_purchased,
            free_tickets_redeemed: value.free_tickets_redeemed,
            free_ticket_eligible: value.free_ticket_eligible,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PlaygroundCheckoutData {
    pub user_id: Uuid,
    pub total_quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub available_coupons: Vec<Uuid>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct TicketSaleSummaryFilter {
    pub start_date: Date,
    pub end_date: Date,
}

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub struct TicketSaleSummary {
    pub sale_date: Date,
    pub tickets_sold: i32,
    pub free_tickets_redeemed: i32,
    pub total_sale: Decimal,
}

impl PlaygroundCheckoutData {
    pub fn new(user_id: Uuid, token_info: TokenInfo, available_coupons: Vec<Uuid>) -> Option<Self> {
        let total_price = Decimal::from_i32(token_info.total_quantity)?.mul(token_info.unit_price);
        Some(Self {
            user_id,
            total_quantity: token_info.total_quantity,
            unit_price: token_info.unit_price,
            available_coupons,
            total_price,
        })
    }
}
