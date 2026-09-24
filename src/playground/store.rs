use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use time::Date;
use uuid::Uuid;

use super::model::{
    AdminPlaygroundPurchaseRow, AdminPlaygroundPurchasesQuery, AdminTokenInfo, ExpiredStatus,
    PlaygroundAccountRow, PlaygroundPurchaseRow, PlaygroundTicketsRow, TicketSaleSummary,
    UserPlaygroundPurchaseHistoryRow, UserPlaygroundPurchasesQuery,
};
use crate::{
    app_error::StoreError,
    db::DBQueryFilter,
    playground::model::{ClaimedTokenInfo, CouponStatus, TokenInfo},
};

pub async fn get_claim_token_info(pool: &PgPool, id: Uuid) -> Result<TokenInfo, StoreError> {
    let result = sqlx::query_as!(
        TokenInfo,
        "SELECT total_quantity,unit_price FROM playground_purchase_tokens where id = $1",
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn get_admin_token_info(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<AdminTokenInfo>, StoreError> {
    let result = sqlx::query_as!(
        AdminTokenInfo,
        r#"SELECT
            total_quantity,
            unit_price,
            claimed_by as "claimed_by?",
            expires_at,
            (expires_at < NOW()) as "expired!"
           FROM playground_purchase_tokens WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

pub async fn get_ticket_sale_summary(
    pool: &PgPool,
    start_date: Date,
    end_date: Date,
) -> Result<Vec<TicketSaleSummary>, StoreError> {
    let rows = sqlx::query_as!(
        TicketSaleSummary,
        r#"
        SELECT
            (created_at AT TIME ZONE 'Asia/Yangon')::date AS "sale_date!",
            SUM(CASE WHEN is_free_redemption = false THEN quantity ELSE 0 END)::int AS "tickets_sold!",
            SUM(CASE WHEN is_free_redemption = true THEN quantity ELSE 0 END)::int AS "free_tickets_redeemed!",
            COALESCE(SUM(CASE WHEN is_free_redemption = false THEN quantity * unit_price ELSE 0 END), 0) AS "total_sale!"
        FROM playground_purchases
        WHERE (created_at AT TIME ZONE 'Asia/Yangon')::date >= $1
            AND (created_at AT TIME ZONE 'Asia/Yangon')::date <= $2
        GROUP BY (created_at AT TIME ZONE 'Asia/Yangon')::date
        ORDER BY (created_at AT TIME ZONE 'Asia/Yangon')::date ASC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_admin_purchases_paginated(
    pool: &PgPool,
    filter: &AdminPlaygroundPurchasesQuery,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdminPlaygroundPurchaseRow>, i64), StoreError> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            pp.id,
            pp.user_id,
            u.username,
            pp.quantity,
            pp.unit_price,
            pp.is_free_redemption,
            pp.created_at
        FROM playground_purchases pp
        JOIN users u ON u.id = pp.user_id
        WHERE 1 = 1
        "#,
    );
    filter.apply_filter(&mut query_builder);
    query_builder.push(" ORDER BY pp.created_at DESC, pp.id DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let rows = query_builder
        .build_query_as::<AdminPlaygroundPurchaseRow>()
        .fetch_all(pool)
        .await?;

    let mut count_builder =
        QueryBuilder::new("SELECT COUNT(*) FROM playground_purchases pp WHERE 1 = 1");
    filter.apply_filter(&mut count_builder);

    let total_items = count_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;

    Ok((rows, total_items))
}

pub async fn get_user_purchases_paginated(
    pool: &PgPool,
    user_id: Uuid,
    filter: &UserPlaygroundPurchasesQuery,
    limit: i64,
    offset: i64,
) -> Result<(Vec<UserPlaygroundPurchaseHistoryRow>, i64), StoreError> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            pp.id,
            pp.user_id,
            pp.quantity,
            pp.unit_price,
            pp.is_free_redemption,
            pp.created_at
        FROM playground_purchases pp
        WHERE pp.user_id =
        "#,
    );
    query_builder.push_bind(user_id);
    filter.apply_filter(&mut query_builder);
    query_builder.push(" ORDER BY pp.created_at DESC, pp.id DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let rows = query_builder
        .build_query_as::<UserPlaygroundPurchaseHistoryRow>()
        .fetch_all(pool)
        .await?;

    let mut count_builder =
        QueryBuilder::new("SELECT COUNT(*) FROM playground_purchases pp WHERE pp.user_id = ");
    count_builder.push_bind(user_id);
    filter.apply_filter(&mut count_builder);

    let total_items = count_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;

    Ok((rows, total_items))
}

pub async fn get_available_user_coupons(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, StoreError> {
    let result = sqlx::query_scalar!(
        "SELECT id FROM playground_coupons WHERE user_id = $1 AND is_used = false;",
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(result)
}

// created_by must be the admin who created the claim token
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
    // 30 min grace period for admin data to check properly
    // before being marked expire
    let marked = sqlx::query!(
        "UPDATE playground_purchase_tokens SET status = 'expired'
         WHERE status = 'pending' AND expires_at + INTERVAL '30 min' < NOW()"
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

pub async fn get_user_playground_account(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<PlaygroundTicketsRow>, StoreError> {
    let result = sqlx::query_as!(PlaygroundTicketsRow,
        "select user_id, tickets_purchased, free_tickets_redeemed,free_ticket_eligible, created_at,updated_at from playground_accounts where user_id = $1", user_id)
        .fetch_optional(pool).await?;
    Ok(result)
}

pub async fn get_user_playground_account_for_update(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<PlaygroundAccountRow, StoreError> {
    let result = sqlx::query_as!(
        PlaygroundAccountRow,
        r#"
    SELECT
        user_id,
        tickets_purchased,
        free_tickets_redeemed,
        created_at,
        updated_at,
        free_ticket_eligible
    FROM playground_accounts
    WHERE user_id = $1
    FOR UPDATE
"#,
        user_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(result)
}

pub async fn insert_ticket_coupon(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), StoreError> {
    sqlx::query!(
        "insert into playground_coupons (user_id,expires_at) VALUES($1,NOW() + INTERVAL '30 day')",
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn deduct_eligible_tickets(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    amount: i32,
) -> Result<PlaygroundAccountRow, StoreError> {
    let result = sqlx::query_as!(
        PlaygroundAccountRow,
        r#"
        UPDATE playground_accounts
        SET free_ticket_eligible = free_ticket_eligible - $1, updated_at = NOW()
        WHERE user_id = $2
        RETURNING user_id,tickets_purchased,free_tickets_redeemed,created_at,updated_at,free_ticket_eligible
    "#,
        amount,
        user_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(result)
}

pub async fn get_coupon_status(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    coupons: &[Uuid],
) -> Result<Vec<CouponStatus>, StoreError> {
    let result = sqlx::query_as!(
        CouponStatus,
        r#"
    SELECT
        id,
        is_used as "is_used!",
        expires_at < NOW() as "is_expired!"
    FROM playground_coupons
    WHERE id = ANY($1)
      AND user_id = $2
    FOR UPDATE"#,
        coupons,
        user_id
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(result)
}

pub async fn update_coupon_status(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    coupons: &[Uuid],
) -> Result<(), StoreError> {
    sqlx::query!(
        r#"
        UPDATE playground_coupons
        SET is_used = true, updated_at = NOW()
        WHERE id = ANY ($1)
        AND user_id = $2
        "#,
        coupons,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_playground_account(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    tickets_purchased: i32,
    free_tickets_redeemed: i32,
) -> Result<PlaygroundTicketsRow, StoreError> {
    let result = sqlx::query_as!(PlaygroundTicketsRow, r#"
        UPDATE playground_accounts
        SET 
            tickets_purchased = tickets_purchased + $1, 
            free_ticket_eligible = free_ticket_eligible + $1,
            free_tickets_redeemed = free_tickets_redeemed + $2,
            updated_at = NOW()
        WHERE user_id = $3
        RETURNING user_id, tickets_purchased,free_tickets_redeemed, free_ticket_eligible,created_at,updated_at
    "#,tickets_purchased,free_tickets_redeemed,user_id).fetch_one(&mut **tx).await?;
    Ok(result)
}

pub async fn insert_playground_purchase(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    quantity: i32,
    is_free_redemption: bool,
    created_by: Uuid,
    unit_price: Decimal,
) -> Result<PlaygroundPurchaseRow, StoreError> {
    let result = sqlx::query_as!(PlaygroundPurchaseRow, r#"
    INSERT INTO playground_purchases (user_id,quantity,is_free_redemption,created_at,created_by,unit_price)
    VALUES($1,$2,$3,NOW(),$4,$5)
    RETURNING id, user_id,quantity,is_free_redemption,created_by,created_at,unit_price
    "#,user_id,quantity,is_free_redemption,created_by,unit_price).fetch_one(&mut **tx).await?;
    Ok(result)
}

pub async fn insert_playground_account(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    tickets_purchased: i32,
    free_tickets_redeemed: i32,
    free_ticket_eligible: i32,
) -> Result<(), StoreError> {
    sqlx::query!(r#"
        INSERT INTO playground_accounts (user_id, tickets_purchased,free_tickets_redeemed,free_ticket_eligible)
        VALUES ($1,$2,$3,$4)
    "#,user_id,tickets_purchased,free_tickets_redeemed,free_ticket_eligible).execute(&mut **tx).await?;
    Ok(())
}
