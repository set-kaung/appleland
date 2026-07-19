use crate::inventory::model::InventoryRow;
use crate::{
    app_error,
    inventory::model::NewInventoryRecord,
};

use sqlx::{Executor, Postgres};

// use when new products are created, to insert a record row
pub async fn insert_new_inventory_record<'e, E>(
    new_inventory_record: NewInventoryRecord,
    executor: E,
) -> Result<InventoryRow, app_error::StoreError>
where
    E: Executor<'e, Database=Postgres>,
{
    let result = sqlx::query_as!(
        InventoryRow,
        r#"
        INSERT INTO inventory (product_id, quantity_received, quantity_remaining, unit_cost, created_by, updated_by)
        VALUES ($1, $2, $3, $4, $5, $5)
        RETURNING *
        "#,
        new_inventory_record.product_id,
        new_inventory_record.quantity_received,
        new_inventory_record.quantity_remaining,
        new_inventory_record.unit_cost,
        new_inventory_record.created_by,
    )
        .fetch_one(executor)
        .await?;

    Ok(result)
}
