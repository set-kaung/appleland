use sqlx::PgPool;
use uuid::Uuid;

use crate::app_error::StoreError;

pub async fn get_today_count(pool: &PgPool, admin_id: Uuid) -> Result<i32, StoreError> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT request_count
        FROM admin_ai_chat_usage
        WHERE admin_id = $1
          AND usage_date = (NOW() AT TIME ZONE 'Asia/Yangon')::date
        "#,
        admin_id
    )
    .fetch_optional(pool)
    .await?
    .unwrap_or(0);

    Ok(count)
}

pub async fn increment_today_count(pool: &PgPool, admin_id: Uuid) -> Result<(), StoreError> {
    sqlx::query!(
        r#"
        INSERT INTO admin_ai_chat_usage (admin_id, usage_date, request_count)
        VALUES ($1, (NOW() AT TIME ZONE 'Asia/Yangon')::date, 1)
        ON CONFLICT (admin_id, usage_date)
        DO UPDATE SET request_count = admin_ai_chat_usage.request_count + 1
        "#,
        admin_id
    )
    .execute(pool)
    .await?;

    Ok(())
}