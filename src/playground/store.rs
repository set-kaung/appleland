use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::model::{ExpiredStatus, PlaygroundTicketsRow};
use crate::{
    app_error::StoreError,
    playground::model::{ClaimedTokenInfo, CouponStatus},
};
pub async fn create_claim_token(
    pool: &PgPool,
    created_by: Uuid,
    total_quantity: i32,
    unit_price: Decimal,
) -> Result<Uuid, StoreError> {
    let result: Uuid = sqlx::query_scalar!(
        "INSERT INTO playground_purchase_tokens (total_quantity, unit_price, created_by, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '1 hour')
         RETURNING id",
        total_quantity,
        unit_price,
        created_by
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn sweep_expired_tokens(pool: &PgPool) -> Result<(u64, u64), StoreError> {
    let mut tx = pool.begin().await?;

    let deleted = sqlx::query!("DELETE FROM playground_purchase_tokens WHERE status = 'expired'")
        .execute(&mut *tx)
        .await?
        .rows_affected();

    let marked = sqlx::query!(
        "UPDATE playground_purchase_tokens SET status = 'expired'
         WHERE status = 'pending' AND expires_at < NOW()"
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    tx.commit().await?;
    Ok((deleted, marked))
}

pub async fn update_claimed_status(
    tx: &mut Transaction<'_, Postgres>,
    claimed_user_id: Uuid,
    token: Uuid,
) -> Result<Option<ClaimedTokenInfo>, StoreError> {
    let claimed = sqlx::query_as!(
        ClaimedTokenInfo,
        r#"UPDATE playground_purchase_tokens
               SET claimed_by = $1, claimed_at = NOW(), status = 'claimed'
               WHERE id = $2 AND status = 'pending' AND expires_at > NOW()
               RETURNING id, claimed_by as "claimed_by!", total_quantity, unit_price, created_by"#,
        claimed_user_id,
        token
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(claimed)
}

pub async fn get_expired_status(
    tx: &mut Transaction<'_, Postgres>,
    token: Uuid,
) -> Result<Option<ExpiredStatus>, StoreError> {
    let row: Option<ExpiredStatus> = sqlx::query_as!(
        ExpiredStatus,
        r#"SELECT (expires_at < NOW()) AS "expired!", status::text AS "status!"
            FROM playground_purchase_tokens WHERE id = $1 FOR UPDATE"#,
        token
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row)
}

pub async fn update_expire_status(
    tx: &mut Transaction<'_, Postgres>,
    token: Uuid,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE playground_purchase_tokens SET status = 'expired'
                    WHERE id = $1 AND status = 'pending'",
        token
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_user_tokens(
    tx: &mut Transaction<'_, Postgres>,
    token_info: ClaimedTokenInfo,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE playground_accounts
            SET tickets_purchased = tickets_purchased + $1, updated_at = NOW()
            WHERE user_id = $2",
        token_info.total_quantity,
        token_info.claimed_by
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// pub async fn claim_token(
//     pool: &PgPool,
//     claimed_user_id: Uuid,
//     token: Uuid,
// ) -> Result<Uuid, StoreError> {
//     let mut tx = pool.begin().await?;
//     let claimed = update_claimed_status(&mut tx, claimed_user_id, token).await?;
//     if let Some(token_info) = claimed {
//         update_user_tokens(&mut tx, token_info).await?;
//         tx.commit().await?;
//         Ok(token)
//     } else {
//         let row = get_expired_status(&mut tx, token).await?;
//         match row {
//             None => Err(StoreError::RecordNotFound),
//             Some(r) if r.expired || r.status == "expired" => {
//                 update_expire_status(&mut tx, token).await?;
//                 tx.commit().await?;
//                 tracing::warn!("claiming on expired token: {token}");
//                 Err(StoreError::TokenExpired)
//             }
//             Some(_) => Err(StoreError::InvalidInput(
//                 "token already claimed".to_string(),
//             )),
//         }
//     }
// }

pub async fn get_user_tickets_info(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<PlaygroundTicketsRow>, StoreError> {
    let result = sqlx::query_as!(PlaygroundTicketsRow,
        "select user_id, tickets_purchased, free_tickets_redeemed, created_at,updated_at from playground_accounts where user_id = $1", user_id)
        .fetch_optional(pool).await?;
    Ok(result)
}

pub async fn insert_ticket_coupon(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), StoreError> {
    let result = sqlx::query!(
        "insert into playground_coupons (user_id,expires_at) VALUES($1,NOW() + INTERVAL '24 hour')",
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn get_coupon_status(
    tx: &mut Transaction<'_, Postgres>,
    coupons: &[Uuid],
) -> Result<Vec<CouponStatus>, StoreError> {
    let result = sqlx::query_as!(
        CouponStatus,
        r#"
    SELECT 
        id, 
        NOT is_used as "is_not_used!", 
        expires_at + INTERVAL '1 minute' > NOW() as "is_expired!"
    FROM playground_coupons
    WHERE id = ANY($1)"#,
        coupons
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(result)
}
