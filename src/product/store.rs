use crate::product::{error::StoreError, model::UpdateProductPayload};

use super::model::{CreateProductPayload, Product};
use sqlx::{Error, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[allow(dead_code)]
pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Product>, Error> {
    sqlx::query_as!(Product, "SELECT * FROM products")
        .fetch_all(pool)
        .await
}

pub async fn fetch_paged(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Product>, i64), Error> {
    let mut conn = pool.acquire().await?;

    let products = sqlx::query_as!(
        Product,
        "SELECT * FROM products LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&mut *conn)
    .await?;

    let (total_items,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(&mut *conn)
        .await?;

    Ok((products, total_items))
}

pub async fn fetch_by_id(pool: &PgPool, id: Uuid) -> Result<Product, Error> {
    sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id)
        .fetch_one(pool)
        .await
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    payload: CreateProductPayload,
) -> Result<Product, Error> {
    sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (sku, name, description, price)
        VALUES ($1, $2, $3, $4)
        RETURNING id, sku, name, description, price, created_at, updated_at
        "#,
        payload.sku,
        payload.name,
        payload.description,
        payload.price
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn update(
    tx: &mut Transaction<'_, Postgres>,
    payload: UpdateProductPayload,
) -> Result<Product, Error> {
    sqlx::query_as!(
        Product,
        r#"UPDATE products
       SET sku = $1, name = $2, description = $3, price = $4, updated_at = NOW()
       WHERE id = $5
       RETURNING id, sku, name, description, price, created_at, updated_at
        "#,
        payload.sku,
        payload.name,
        payload.description,
        payload.price,
        payload.id
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn delete(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, StoreError> {
    let result = sqlx::query!(r#"DELETE from products WHERE id = $1"#, id)
        .execute(&mut **tx)
        .await?;
    let affected = result.rows_affected();

    if affected > 1 {
        return Err(StoreError::SafetyTrigger(affected));
    }

    Ok(affected == 1)
}
