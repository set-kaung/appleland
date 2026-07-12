use sqlx::PgPool;
use uuid::Uuid;
use crate::app_error::StoreError;
use crate::sales::model::{ AdminSaleDetails, SaleRow,SaleItem};

pub async fn fetch_all_paginated(pool:&PgPool,limit: i64,offset: i64,) -> Result<(Vec<SaleRow>,i64),StoreError> {
    let rows = sqlx::query_as!(SaleRow,r#"
    SELECT id,total_amount,created_at from sales
    LIMIT $1 OFFSET $2
    "#,limit,offset).fetch_all(pool).await?;

    let total_items = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;

   Ok((rows, total_items.unwrap_or(0)))
}

pub async fn fetch_sale_detail_by_id(pool:&PgPool,sale_id:Uuid) -> Result<AdminSaleDetails,StoreError>{
    let sale_row = sqlx::query_as!(SaleRow,r#"
        SELECT id,total_amount,created_at from sales
    "#).fetch_one(pool).await?;
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
    Ok(AdminSaleDetails{id:sale_row.id, total_amount: sale_row.total_amount, created_at: sale_row.created_at, sale_items })
}