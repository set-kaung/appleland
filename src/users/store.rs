use crate::{
    app_error::StoreError,
    users::model::{UserContactRow, UserRow},
};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn find_user_id_by_supabase_id(
    pool: &PgPool,
    supabase_id: Uuid,
) -> Result<Option<Uuid>, StoreError> {
    let result = sqlx::query_scalar!(
        r#"SELECT id FROM users WHERE supabase_id = $1"#,
        supabase_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(result)
}

pub async fn insert_into_users_supabase(
    tx: &mut Transaction<'_, Postgres>,
    supabase_id: Uuid,
    username: String,
) -> Result<Uuid, StoreError> {
    let result = sqlx::query_scalar!(
        r#"INSERT INTO users (supabase_id, username) VALUES($1,$2) RETURNING id"#,
        supabase_id,
        username
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(result)
}

pub async fn insert_into_user_contact(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    address_line_1: String,
    phone_number: String,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO user_contact (address_line_1, phone_number, user_id) VALUES ($1,$2,$3)",
        address_line_1,
        phone_number,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn fetch_user_contact_by_user_id<'e, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Option<UserContactRow>, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query_as!(
        UserContactRow,
        r#"
        SELECT address_line_1, phone_number
        FROM user_contact
        WHERE user_id = $1
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(executor)
    .await?;

    Ok(result)
}

pub async fn get_all_users_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<UserRow>, i64), StoreError> {
    let result = sqlx::query_as!(
        UserRow,
        r#"
        SELECT users.id, users.username, uc.address_line_1, uc.phone_number
        FROM users
        LEFT JOIN LATERAL (
            SELECT address_line_1, phone_number
            FROM user_contact
            WHERE user_id = users.id
            ORDER BY created_at ASC, id ASC
            LIMIT 1
        ) uc ON TRUE
        ORDER BY users.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await?;

    Ok((result, total.unwrap_or(0)))
}
