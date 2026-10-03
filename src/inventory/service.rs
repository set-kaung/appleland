use super::model::{
    CreateAdjustmentPayload, InventoryAdjustment, InventoryBatchInfo, InventoryBatchRow, StockLevel,
};
use super::store;
use crate::app_error::StoreError;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn get_stock_levels(pool: &PgPool, limit: i64) -> Result<Vec<StockLevel>, StoreError> {
    let rows = store::get_product_stock_levels(pool, limit).await?;

    Ok(rows
        .into_iter()
        .map(|row| StockLevel {
            product_name: row.product_name,
            product_id: row.product_id,
            quantity_in_stock: row.quantity_in_stock,
            is_low_stock: row.quantity_in_stock <= 10,
        })
        .collect())
}

pub async fn create_adjustment(
    pool: &PgPool,
    payload: &CreateAdjustmentPayload,
    admin_id: Uuid,
) -> Result<InventoryAdjustment, StoreError> {
    if payload.quantity_change == 0 {
        return Err(StoreError::InvalidInput(
            "quantity_change must not be zero".to_string(),
        ));
    }

    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    if payload.quantity_change > 0 {
        apply_addition(&mut tx, payload).await?;
    } else {
        apply_deduction(&mut tx, payload).await?;
    }

    let row = store::insert_inventory_adjustment(
        &mut tx,
        payload.product_id,
        payload.inventory_id,
        payload.quantity_change,
        payload.reason,
        admin_id,
        payload.note.as_deref(),
    )
    .await?;
    tx.commit().await?;
    Ok(row.into())
}

async fn apply_addition(
    tx: &mut Transaction<'_, Postgres>,
    payload: &CreateAdjustmentPayload,
) -> Result<(), StoreError> {
    let Some(inventory_id) = payload.inventory_id else {
        return Err(StoreError::InvalidInput(
            "inventory_id is required for stock addition fixes.".to_string(),
        ));
    };

    let batch = store::lock_inventory_batch(tx, payload.product_id, inventory_id).await?;
    if batch
        .quantity_remaining
        .checked_add(payload.quantity_change)
        .is_none()
    {
        return Err(StoreError::InvalidInput(
            "resulting inventory quantity is too large".to_string(),
        ));
    }

    let rows_affected =
        store::update_inventory_batch_quantity(tx, inventory_id, payload.quantity_change).await?;
    if rows_affected != 1 {
        return Err(StoreError::RecordNotFound);
    }

    Ok(())
}

async fn apply_deduction(
    tx: &mut Transaction<'_, Postgres>,
    payload: &CreateAdjustmentPayload,
) -> Result<(), StoreError> {
    let requested = payload
        .quantity_change
        .checked_abs()
        .ok_or_else(|| StoreError::InvalidInput("quantity_change is invalid".to_string()))?;

    match payload.inventory_id {
        Some(inventory_id) => {
            let batch = store::lock_inventory_batch(tx, payload.product_id, inventory_id).await?;
            if batch.quantity_remaining < requested {
                return Err(StoreError::InsufficientInventory(
                    payload.product_id,
                    requested,
                    batch.quantity_remaining,
                ));
            }
            store::deduct_inventory_batch(tx, &[(inventory_id, requested)]).await?;
        }
        None => {
            let batches = store::lock_fifo_inventory_batches(tx, payload.product_id).await?;
            let deductions = build_fifo_deductions(payload.product_id, &batches, requested)?;
            store::deduct_inventory_batch(tx, &deductions).await?;
        }
    }

    Ok(())
}

pub(super) fn build_fifo_deductions(
    product_id: Uuid,
    batches: &[InventoryBatchRow],
    requested: i32,
) -> Result<Vec<(Uuid, i32)>, StoreError> {
    let available = batches
        .iter()
        .try_fold(0_i32, |total, batch| total.checked_add(batch.quantity_remaining))
        .ok_or_else(|| StoreError::InvalidInput("available quantity overflow".to_string()))?;

    if available < requested {
        return Err(StoreError::InsufficientInventory(
            product_id,
            requested,
            available,
        ));
    }

    let mut remaining = requested;
    let mut deductions = Vec::new();
    for batch in batches {
        if remaining == 0 {
            break;
        }
        let deducted = remaining.min(batch.quantity_remaining);
        if deducted == 0 {
            continue;
        }
        deductions.push((batch.id, deducted));
        remaining -= deducted;
    }

    Ok(deductions)
}

pub async fn get_product_batches(
    pool: &PgPool,
    product_id: Uuid,
    available_only: bool,
) -> Result<Vec<InventoryBatchInfo>, StoreError> {
    let rows = store::get_product_batches(pool, product_id, available_only).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
