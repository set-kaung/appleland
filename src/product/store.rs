use crate::product::model::UpdateProductPayload;

use super::model::{CreateProductPayload, Product};
use sqlx::{Error, Executor, PgPool, Postgres};
use uuid::Uuid;

#[allow(dead_code)]
pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Product>, Error> {
    sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(pool)
        .await
}

pub async fn fetch_paged(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Product>, i64), Error> {
    let mut conn = pool.acquire().await?;

    let products = sqlx::query_as::<_, Product>("SELECT * FROM products LIMIT $1 OFFSET $2")
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await?;

    let (total_items,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(&mut *conn)
        .await?;

    Ok((products, total_items))
}

pub async fn fetch_by_id(pool: &PgPool, id: Uuid) -> Result<Product, Error> {
    sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn create<'e, E>(pool: E, payload: CreateProductPayload) -> Result<Product, Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (sku, name, description, price)
        VALUES ($1, $2, $3, $4)
        RETURNING id, sku, name, description, price, created_at, updated_at
        "#,
    )
    .bind(payload.sku)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.price)
    .fetch_one(pool)
    .await
}

pub async fn update<'e, E>(pool: E, payload: UpdateProductPayload) -> Result<Product, Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, Product>(
        r#"UPDATE products 
       SET sku = $1, name = $2, description = $3, price = $4, updated_at = NOW()
       WHERE id = $5
       RETURNING id, sku, name, description, price, created_at, updated_at
        "#,
    )
    .bind(payload.sku)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.price)
    .bind(payload.id)
    .fetch_one(pool)
    .await
}
