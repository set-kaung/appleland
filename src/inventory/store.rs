use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::app_error;

// use when new products are created, to insert a record row
pub async fn insert_inventory_row(
    product_id: Uuid,
    admin_id: Uuid,
    quantity: i32,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), app_error::StoreError> {
    let result = sqlx::query!(
        r#"insert into inventory (product_id, quantity, created_by, updated_by)
        VALUES ($1, $2, $3, $3)
        "#,
        product_id,
        quantity,
        admin_id,
    )
    .execute(&mut **tx)
    .await?;
    if result.rows_affected() != 1 {
        return Err(app_error::StoreError::NoRowsAffected);
    }
    Ok(())
}
