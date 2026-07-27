use crate::app_error::StoreError;
use crate::sales::model::{AdminSaleDetails, SaleItem, SaleProductDTO, SaleRow};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
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
    let sale_row = sqlx::query_as!(
        SaleRow,
        r#"
        SELECT id, total_amount, created_at FROM sales WHERE id = $1
    "#,
        sale_id
    )
    .fetch_one(pool)
    .await?;
    let sale_items = sqlx::query_as!(
        SaleItem,
        r#"SELECT si.id, si.product_id, p.name as product_name, si.sale_id, si.quantity, si.selling_price,si.cost_price, si.created_at
           FROM sale_items si
           JOIN products p on p.id = si.product_id
           WHERE sale_id = $1"#,
        sale_id
    )
        .fetch_all(pool)
        .await?;
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
) -> Result<Vec<(Uuid, Decimal)>, StoreError> {
    let rows = sqlx::query_as!(
        PriceRow,
        "SELECT id, selling_price FROM products WHERE id = ANY($1)",
        product_ids
    )
        .fetch_all(&mut **tx)
        .await?;

    Ok(rows.into_iter().map(|r| (r.id, r.selling_price)).collect())
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
