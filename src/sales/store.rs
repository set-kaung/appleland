use crate::app_error::StoreError;
use crate::db::DBQueryFilter;
use crate::sales::model::{
    AdminSaleDetails, SaleItem, SaleProductDTO, SaleRow, SaleSummaryFilter, SaleSummaryRow,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn fetch_all_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SaleRow>, i64), StoreError> {
    let rows = sqlx::query_as!(
        SaleRow,
        r#"
    SELECT id,total_amount,created_at from sales
    ORDER BY created_at DESC, id DESC
    LIMIT $1 OFFSET $2
    "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let total_items = sqlx::query_scalar!("SELECT COUNT(*) FROM sales")
        .fetch_one(pool)
        .await?;

    Ok((rows, total_items.unwrap_or(0)))
}

pub async fn fetch_sale_detail_by_id(
    pool: &PgPool,
    sale_id: Uuid,
) -> Result<AdminSaleDetails, StoreError> {
    let mut tx = pool.begin().await?;

    let sale_row = sqlx::query_as!(
        SaleRow,
        r#"
        SELECT id, total_amount, created_at FROM sales WHERE id = $1
    "#,
        sale_id
    )
    .fetch_one(&mut *tx)
    .await?;
    let sale_items = sqlx::query_as!(
        SaleItem,
        r#"SELECT si.id, si.product_id, p.name as product_name, si.sale_id, si.quantity, si.selling_price,si.cost_price, si.created_at
           FROM sale_items si
           JOIN products p on p.id = si.product_id
           WHERE sale_id = $1"#,
        sale_id
    )
        .fetch_all(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(AdminSaleDetails {
        id: sale_row.id,
        total_amount: sale_row.total_amount,
        created_at: sale_row.created_at,
        sale_items,
    })
}

pub async fn insert_sale(
    tx: &mut Transaction<'_, Postgres>,
    total_amount: Decimal,
) -> Result<Uuid, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO sales (total_amount) VALUES($1) RETURNING id",
        total_amount
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(id)
}

pub async fn insert_admin_sale(
    tx: &mut Transaction<'_, Postgres>,
    sale_id: Uuid,
    admin_id: Uuid,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO admin_sales (sale_id,admin_id) VALUES ($1,$2)",
        sale_id,
        admin_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn fetch_selling_prices(
    tx: &mut Transaction<'_, Postgres>,
    product_ids: &[Uuid],
) -> Result<HashMap<Uuid, Decimal>, StoreError> {
    let rows = sqlx::query_as!(
        PriceRow,
        "SELECT id, selling_price FROM products WHERE id = ANY($1) AND is_deleted IS FALSE",
        product_ids
    )
    .fetch_all(&mut **tx)
    .await?;

    let found: HashMap<Uuid, Decimal> = rows.into_iter().map(|r| (r.id, r.selling_price)).collect();

    // for all client requestd product ids
    // check if any is missing in products returned from db
    let missing: Vec<Uuid> = product_ids
        .iter()
        .filter(|id| !found.contains_key(id))
        .copied()
        .collect();

    // if missing is not empty
    // invalid product ids present, return error
    // else, all good, return the result
    if !missing.is_empty() {
        return Err(StoreError::InvalidInput(format!(
            "products not found: {}",
            missing
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    Ok(found)
}

#[derive(sqlx::FromRow)]
struct PriceRow {
    id: Uuid,
    selling_price: Decimal,
}

pub async fn insert_sale_items(
    tx: &mut Transaction<'_, Postgres>,
    sale_id: Uuid,
    sale_products: Vec<SaleProductDTO>,
) -> Result<(), StoreError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO sale_items (sale_id, product_id, quantity, selling_price, cost_price, inventory_id) ",
    );

    qb.push_values(sale_products, |mut row, item| {
        row.push_bind(sale_id)
            .push_bind(item.product_id)
            .push_bind(item.quantity)
            .push_bind(item.selling_price)
            .push_bind(item.cost_price)
            .push_bind(item.inventory_id);
    });

    qb.build().execute(&mut **tx).await?;
    Ok(())
}

pub async fn get_sale_summary(
    pool: &PgPool,
    filter: SaleSummaryFilter,
) -> Result<Vec<SaleSummaryRow>, StoreError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
        WITH filtered_sales AS (
            SELECT
                s.id,
                s.created_at::date AS sale_date,
                s.total_amount,
                (a.id IS NULL) AS is_online_sale
            FROM sales s
            LEFT JOIN admin_sales a ON a.sale_id = s.id
            WHERE 1=1",
    );
    filter.apply_filter(&mut qb);
    qb.push(
        "
        ),
        item_counts AS (
            SELECT
                sale_id,
                SUM(quantity)::INT4 AS items_sold,
                SUM(quantity * cost_price) AS total_cost
            FROM sale_items
            GROUP BY sale_id
        )
        SELECT
            fs.sale_date,
            SUM(fs.total_amount) AS total_sale,
            fs.is_online_sale,
            COUNT(*)::INT4 AS transactions,
            COALESCE(SUM(ic.items_sold), 0)::INT4 AS items_sold,
            COALESCE(SUM(ic.total_cost), 0) AS total_cost,
            SUM(fs.total_amount) - COALESCE(SUM(ic.total_cost), 0) AS margin
        FROM filtered_sales fs
        LEFT JOIN item_counts ic ON ic.sale_id = fs.id
        GROUP BY fs.sale_date, fs.is_online_sale
        ",
    );
    let query = qb.build_query_as::<SaleSummaryRow>();

    let result = query.fetch_all(pool).await?;

    Ok(result)
}
