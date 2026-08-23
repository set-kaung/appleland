use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use super::model::PlaygroundTicketsRow;
use crate::app_error::StoreError;
pub async fn create_claim_token(
    pool: &PgPool,
    created_by: Uuid,
    quantity: i32,
) -> Result<Uuid, StoreError> {
    let result: Uuid = sqlx::query_scalar!(
        "INSERT INTO playground_purchase_tokens (quantity, created_by, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 hour')
         RETURNING id",
        quantity,
        created_by
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn sweep_expired_tokens(pool: &PgPool) -> Result<(u64, u64), StoreError> {
    let deleted = sqlx::query!("DELETE FROM playground_purchase_tokens WHERE status = 'expired'")
        .execute(pool)
        .await?
        .rows_affected();

    let marked = sqlx::query!(
        "UPDATE playground_purchase_tokens SET status = 'expired'
         WHERE status = 'pending' AND expires_at < NOW()"
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok((deleted, marked))
}


pub async fn claim_token(
    pool: &PgPool,
    claimed_user_id: Uuid,
    token: Uuid,
) -> Result<Uuid, StoreError> {
    let claimed = sqlx::query_scalar!(
        r#"UPDATE playground_purchase_tokens
           SET claimed_by = $1, claimed_at = NOW(), status = 'claimed'
           WHERE id = $2 AND status = 'pending' AND expires_at > NOW()
           RETURNING id"#,
        claimed_user_id,
        token
    )
    .fetch_optional(pool)
    .await?;

    if let Some(id) = claimed {
        return Ok(id);
    }
    #[derive(FromRow)]
    struct ExpiredStatus{
        expired: bool,
        status: String
    }

    let row:Option<ExpiredStatus> = sqlx::query_as!(ExpiredStatus,
        r#"SELECT (expires_at < NOW()) AS "expired!", status::text AS "status!"
           FROM playground_purchase_tokens WHERE id = $1"#,
        token
    )
    .fetch_optional(pool)
    .await?;

    match row {
        None => Err(StoreError::RecordNotFound),
        Some(r) if r.expired || r.status == "expired" => {
            sqlx::query!(
                "UPDATE playground_purchase_tokens SET status = 'expired'
                 WHERE id = $1 AND status = 'pending'",
                token
            )
            .execute(pool)
            .await?;
            Err(StoreError::TokenExpired(token.to_string()))
        }
        Some(_) => Err(StoreError::InvalidInput("token already claimed".to_string())),
    }
}

pub async fn get_user_tickets_info(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PlaygroundTicketsRow, StoreError> {
    let result = sqlx::query_as!(PlaygroundTicketsRow,
        "select user_id, tickets_purchased, free_tickets_redeemed, created_at,updated_at from playground_accounts where user_id = $1", user_id)
        .fetch_one(pool).await?;
    Ok(result)
}
