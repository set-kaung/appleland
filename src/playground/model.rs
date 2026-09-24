use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::ops::Mul;
use time::{Date, OffsetDateTime};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::db::DBQueryFilter;

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
    pub user_id: Option<Uuid>,
    pub total_quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub available_coupons: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlaygroundCheckoutDataAdmin {
    pub claimed_by: Option<Uuid>,
    pub total_quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Option<Decimal>,
    pub available_coupons: Vec<Uuid>,
    pub expires_at: OffsetDateTime,
    pub expired: bool,
}

#[derive(Debug, FromRow)]
pub struct AdminTokenInfo {
    pub total_quantity: i32,
    pub unit_price: Decimal,
    pub claimed_by: Option<Uuid>,
    pub expires_at: OffsetDateTime,
    pub expired: bool,
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
    pub fn new(
        user_id: Option<Uuid>,
        token_info: TokenInfo,
        available_coupons: Vec<Uuid>,
    ) -> Option<Self> {
        let coupons_redeemed = available_coupons
            .len()
            .min(token_info.total_quantity as usize) as i32;
        let charged_quantity = token_info.total_quantity - coupons_redeemed;
        let total_price = Decimal::from_i32(charged_quantity)?.mul(token_info.unit_price);
        Some(Self {
            user_id,
            total_quantity: token_info.total_quantity,
            unit_price: token_info.unit_price,
            available_coupons,
            total_price,
        })
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct AdminPlaygroundPurchasesQuery {
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub is_free_redemption: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl AdminPlaygroundPurchasesQuery {
    pub fn get_limit_and_offset(&self) -> (i64, i64, i64) {
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1).saturating_mul(page_size);

        (page_size, offset, page)
    }
}

impl DBQueryFilter<Postgres> for AdminPlaygroundPurchasesQuery {
    fn apply_filter(&self, query_builder: &mut QueryBuilder<Postgres>) {
        if let Some(start_date) = self.start_date {
            query_builder.push(" AND (pp.created_at AT TIME ZONE 'Asia/Yangon')::date >= ");
            query_builder.push_bind(start_date);
        }
        if let Some(end_date) = self.end_date {
            query_builder.push(" AND (pp.created_at AT TIME ZONE 'Asia/Yangon')::date <= ");
            query_builder.push_bind(end_date);
        }
        if let Some(is_free_redemption) = self.is_free_redemption {
            query_builder.push(" AND pp.is_free_redemption = ");
            query_builder.push_bind(is_free_redemption);
        }
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct UserPlaygroundPurchasesQuery {
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub is_free_redemption: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl UserPlaygroundPurchasesQuery {
    pub fn get_limit_and_offset(&self) -> (i64, i64, i64) {
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1).saturating_mul(page_size);

        (page_size, offset, page)
    }
}

impl DBQueryFilter<Postgres> for UserPlaygroundPurchasesQuery {
    fn apply_filter(&self, query_builder: &mut QueryBuilder<Postgres>) {
        if let Some(start_date) = self.start_date {
            query_builder.push(" AND (pp.created_at AT TIME ZONE 'Asia/Yangon')::date >= ");
            query_builder.push_bind(start_date);
        }
        if let Some(end_date) = self.end_date {
            query_builder.push(" AND (pp.created_at AT TIME ZONE 'Asia/Yangon')::date <= ");
            query_builder.push_bind(end_date);
        }
        if let Some(is_free_redemption) = self.is_free_redemption {
            query_builder.push(" AND pp.is_free_redemption = ");
            query_builder.push_bind(is_free_redemption);
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserPlaygroundPurchaseHistoryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_free_redemption: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPlaygroundPurchaseHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_free_redemption: bool,
    pub line_total: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<UserPlaygroundPurchaseHistoryRow> for UserPlaygroundPurchaseHistory {
    fn from(value: UserPlaygroundPurchaseHistoryRow) -> Self {
        let line_total = if value.is_free_redemption {
            Decimal::ZERO
        } else {
            value.unit_price * Decimal::from(value.quantity)
        };
        Self {
            id: value.id,
            user_id: value.user_id,
            quantity: value.quantity,
            unit_price: value.unit_price,
            is_free_redemption: value.is_free_redemption,
            line_total,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct AdminPlaygroundPurchaseRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_free_redemption: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminPlaygroundPurchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_free_redemption: bool,
    pub line_total: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<AdminPlaygroundPurchaseRow> for AdminPlaygroundPurchase {
    fn from(value: AdminPlaygroundPurchaseRow) -> Self {
        let line_total = if value.is_free_redemption {
            Decimal::ZERO
        } else {
            value.unit_price * Decimal::from(value.quantity)
        };
        Self {
            id: value.id,
            user_id: value.user_id,
            username: value.username,
            quantity: value.quantity,
            unit_price: value.unit_price,
            is_free_redemption: value.is_free_redemption,
            line_total,
            created_at: value.created_at,
        }
    }
}
