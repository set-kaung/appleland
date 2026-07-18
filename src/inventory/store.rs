use sqlx::{Postgres, Transaction};


use crate::app_error::StoreError;
use crate::{
    app_error,
    inventory::model::NewInventoryRecord,
};


// use when new products are created, to insert a record row
pub async fn insert_new_inventory_record(
    new_inventory_record: NewInventoryRecord,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), app_error::StoreError> {
    let result = sqlx::query!(
        r#"insert into inventory (product_id, quantity_received,quantity_remaining,unit_cost, created_by, updated_by)
        VALUES ($1, $2, $3,$4, $5,$5)
        "#,
        new_inventory_record.product_id,
        new_inventory_record.quantity_received,
        new_inventory_record.quantity_remaining,
        new_inventory_record.unit_cost,
        new_inventory_record.created_by,
    )
        .execute(&mut **tx)
        .await?;
    if result.rows_affected() != 1 {
        return Err(StoreError::RecordNotFound);
    }
    Ok(())
}

