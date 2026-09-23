use super::model::StockLevel;
use super::store;
use crate::app_error::StoreError;
use sqlx::PgPool;

pub async fn get_stock_levels(
    pool: &PgPool,
    limit: i64,
    low_stock_threshold: i32,
) -> Result<Vec<StockLevel>, StoreError> {
    let rows = store::get_product_stock_levels(pool, limit).await?;

    Ok(rows
        .into_iter()
        .map(|row| StockLevel {
            product_name: row.product_name,
            product_id: row.product_id,
            quantity_in_stock: row.quantity_in_stock,
            is_low_stock: row.quantity_in_stock <= low_stock_threshold,
        })
        .collect())
}
