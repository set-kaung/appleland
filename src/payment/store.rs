use sqlx::PgPool;
use std::time::Duration;

use crate::app_error::StoreError;

/// Returns `true` if `nonce` was newly consumed (first time seen),
/// `false` if it was already used (replay).
pub async fn consume_nonce(pool: &PgPool, nonce: &str) -> Result<bool, StoreError> {
    let row = sqlx::query!(
        r#"INSERT INTO payment_webhook_nonces (nonce) VALUES($1)
           ON CONFLICT (nonce) DO NOTHING
           RETURNING nonce"#,
        nonce
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

/// Deletes nonces older than `retention`. Returns the number of rows removed.
pub async fn delete_expired_nonces(pool: &PgPool, retention: Duration) -> Result<u64, StoreError> {
    let interval = format!("{} seconds", retention.as_secs());
    let result = sqlx::query!(
        r#"DELETE FROM payment_webhook_nonces
           WHERE consumed_at < now() - ($1::text)::interval"#,
        interval
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}