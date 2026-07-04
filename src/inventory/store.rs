use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    app_error,
    inventory::model::{Inventory, UpdateInventory},
};

// use when new products are created, to insert a record row
pub async fn insert_new_inventory_record(
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

pub async fn update(
    update_inventory_data: UpdateInventory,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Inventory, app_error::StoreError> {
    sqlx::query_as!(
        Inventory,
        r#"update inventory 
    set quantity = $1, updated_by = $2, updated_at = NOW()
    where product_id = $3
    RETURNING id, product_id, quantity, created_by, updated_by, created_at, updated_at
    "#,
        update_inventory_data.quantity,
        update_inventory_data.update_by,
        update_inventory_data.product_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(app_error::StoreError::Database)
}
