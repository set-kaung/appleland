use super::model::{AdminRole, AdminRow};
use crate::app_error;

pub async fn get_admin_by_username(
    pool: &sqlx::PgPool,
    username: String,
) -> Result<AdminRow, app_error::StoreError> {
    sqlx::query_as!(
        AdminRow,
        "
    SELECT id,username,password_hash,role as \"role: AdminRole\",created_at,updated_at
    FROM admins
    WHERE username = $1",
        username
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => app_error::StoreError::RecordNotFound,
        _ => {
            tracing::debug!("{}", e);
            app_error::StoreError::Database(e)
        }
    })
}
