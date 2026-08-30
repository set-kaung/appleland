use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct CreateClaimTokenPayload {
    pub purchased_quantity: i32,
    pub free_quantity: i32,
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
