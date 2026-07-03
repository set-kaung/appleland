use super::model::Product;
use crate::product::model::UpdateProduct;
use crate::{app_error, product::model::InsertProduct};
use sqlx::{Error, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Product>, app_error::StoreError> {
    sqlx::query_as!(Product, "SELECT * FROM products")
        .fetch_all(pool)
        .await
        .map_err(app_error::StoreError::Database)
}

pub async fn fetch_paged(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Product>, i64), app_error::StoreError> {
    let products = sqlx::query_as!(
        Product,
        "SELECT * FROM products LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(app_error::StoreError::Database)?;

    let total_items = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await
        .map_err(app_error::StoreError::Database)?;

    Ok((products, total_items.unwrap_or(0)))
}

pub async fn fetch_by_id(pool: &PgPool, id: Uuid) -> Result<Product, Error> {
    sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id)
        .fetch_one(pool)
        .await
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    payload: InsertProduct,
) -> Result<Product, Error> {
    sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (barcode, name, description,category,sub_category,price,is_active,created_by,updated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7,$8,$8)
        RETURNING id, barcode, name, description,category,sub_category, price, is_active, created_at, updated_at, created_by,updated_by
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category,
        payload.sub_category,
        payload.price,
        payload.is_active,
        payload.created_by
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn update(
    tx: &mut Transaction<'_, Postgres>,
    payload: UpdateProduct,
) -> Result<Product, Error> {
    sqlx::query_as!(
        Product,
        r#"UPDATE products
       SET barcode = $1, name = $2, description = $3,category = $4,sub_category = $5, price = $6, is_active = $7,updated_by= $8, updated_at = NOW()
       WHERE id = $9
       RETURNING id,barcode, name, description,category, sub_category, price, is_active, created_at, updated_at,created_by,updated_by
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category,
        payload.sub_category,
        payload.price,
        payload.is_active,
        payload.updated_by,
        payload.id,
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn delete(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, app_error::StoreError> {
    let result = sqlx::query!(r#"DELETE from products WHERE id = $1"#, id)
        .execute(&mut **tx)
        .await?;
    let affected = result.rows_affected();

    if affected > 1 {
        return Err(app_error::StoreError::SafetyTrigger(affected));
    }

    if affected == 0 {
        return Err(app_error::StoreError::NoRows);
    }

    Ok(affected == 1)
}
