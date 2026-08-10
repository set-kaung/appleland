use crate::inventory::model::InventoryRow;
use crate::{
    app_error,
    inventory::model::NewInventoryRecord,
};

use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

// use when new products are created, to insert a record row
pub async fn insert_new_inventory_record<'e, E>(
    executor: E,
    new_inventory_record: NewInventoryRecord,
) -> Result<InventoryRow, app_error::StoreError>
where
    E: Executor<'e, Database=Postgres>,
{
    let result = sqlx::query_as!(
        InventoryRow,
        r#"
        INSERT INTO inventory (product_id, quantity_received, quantity_remaining, unit_cost, created_by, updated_by)
        VALUES ($1, $2, $3, $4, $5, $5)
        RETURNING id, product_id, quantity_received, quantity_remaining, unit_cost, created_by, updated_by, created_at, updated_at
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

pub async fn fetch_inventory_fifo_batch(
    tx: &mut Transaction<'_, Postgres>,
    product_ids: &[Uuid],
) -> Result<Vec<InventoryRow>, app_error::StoreError> {
    let rows = sqlx::query_as!(
        InventoryRow,
        r#"
        SELECT id, product_id, quantity_received, quantity_remaining, unit_cost, created_by, updated_by, created_at, updated_at
        FROM inventory
        WHERE product_id = ANY($1) AND quantity_remaining > 0
        ORDER BY product_id, created_at ASC, id ASC
        "#,
        product_ids
    )
        .fetch_all(&mut **tx)
        .await?;

    Ok(rows)
}

pub async fn deduct_inventory_batch(
    tx: &mut Transaction<'_, Postgres>,
    deductions: &[(Uuid, i32)],
) -> Result<(), app_error::StoreError> {
    let mut qb = sqlx::QueryBuilder::new(
        "UPDATE inventory AS i SET quantity_remaining = i.quantity_remaining - d.deduct, updated_at = NOW() FROM (VALUES ",
    );

    let mut separated = qb.separated(", ");
    for (id, qty) in deductions {
        separated.push("(");
        separated.push_bind_unseparated(*id);
        separated.push_unseparated("::uuid, ");
        separated.push_bind_unseparated(*qty);
        separated.push_unseparated(")");
    }

    qb.push(") AS d(id, deduct) WHERE i.id = d.id AND i.quantity_remaining >= d.deduct");

    let result = qb.build().execute(&mut **tx).await?;

    if result.rows_affected() != deductions.len() as u64 {
        return Err(app_error::StoreError::UnexpectedRowsAffected(result.rows_affected()));
    }

    Ok(())
}
