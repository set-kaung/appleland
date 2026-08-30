use sqlx::PgPool;
use uuid::Uuid;

use super::store;
use crate::app_error::StoreError;

pub async fn is_product_perishable(pool: &PgPool, id: Uuid) -> Result<bool, StoreError> {
    store::fetch_product_perishability(pool, id).await
}
