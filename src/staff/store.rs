use crate::app_error::StoreError;
use crate::auth::admin::model::AdminRole;
use crate::staff::model::AdminStaff;
use sqlx::PgPool;

pub async fn get_admins(pool: &PgPool) -> Result<Vec<AdminStaff>, StoreError> {
    let result = sqlx::query_as!(
        AdminStaff,
        "SELECT id,username,role as \"role: AdminRole\" from admins"
    )
    .fetch_all(pool)
    .await?;

    Ok(result)
}
