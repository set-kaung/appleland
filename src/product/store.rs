use crate::app_error::StoreError;
use crate::product::model::{ProductRow, UpdateProduct};
use crate::{app_error, product::model::InsertProduct};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<ProductRow>, app_error::StoreError> {
    sqlx::query_as!(
        ProductRow,
        r#"
            SELECT p.*, c1.name as category, c2.name as sub_category, i.quantity as quantity_in_stock
            FROM products p
            JOIN inventory i on p.id = i.product_id
            JOIN categories c1 on c1.id = p.category_id
            LEFT JOIN categories c2 on c2.id = p.sub_category_id
            "#
    )
    .fetch_all(pool)
    .await.map_err(StoreError::Database)
}

pub async fn fetch_paged(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    is_active: bool,
) -> Result<(Vec<ProductRow>, i64), app_error::StoreError> {
    let products = sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.*,c1.name as category, c2.name as sub_category, i.quantity as quantity_in_stock
        FROM products p
        JOIN inventory i on p.id = i.product_id
        JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        WHERE p.is_active = $3
        LIMIT $1 OFFSET $2"#,
        limit,
        offset,
        is_active,
    )
    .fetch_all(pool)
    .await?;

    let total_items = sqlx::query_scalar!(
    "SELECT COUNT(*) FROM products WHERE is_active = $1",
    is_active
)
        .fetch_one(pool)
        .await?;

    Ok((products, total_items.unwrap_or(0)))
}

pub async fn fetch_paged_admin(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ProductRow>, i64), app_error::StoreError> {
    let products = sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.*,c1.name as category, c2.name as sub_category, COALESCE(i.quantity, 0) as "quantity_in_stock!"
        FROM products p
        LEFT JOIN inventory i on p.id = i.product_id
        JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        LIMIT $1 OFFSET $2"#,
        limit,
        offset,
    )
        .fetch_all(pool)
        .await?;

    let total_items = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;

    Ok((products, total_items.unwrap_or(0)))
}


pub async fn fetch_by_id<'e, E>(executor: E, id: Uuid) -> Result<ProductRow, app_error::StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.*,c1.name as category,c2.name as sub_category, i.quantity as quantity_in_stock
        FROM products p
        JOIN inventory i on p.id = i.product_id
        JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        WHERE p.id = $1"#,
        id
    )
    .fetch_one(executor)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => app_error::StoreError::RecordNotFound,
        e => app_error::StoreError::Database(e),
    })
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    payload: InsertProduct,
) -> Result<Uuid, app_error::StoreError> {
    let product_id = sqlx::query_scalar!(
        r#"
        INSERT INTO products (barcode, name, description,category_id,sub_category_id,price,is_active,created_by,updated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7,$8,$8)
        RETURNING
            id
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category_id,
        payload.sub_category_id,
        payload.price,
        payload.is_active,
        payload.created_by
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(product_id)
}

pub async fn update(
    tx: &mut Transaction<'_, Postgres>,
    payload: UpdateProduct,
) -> Result<ProductRow, app_error::StoreError> {
    let product_id = sqlx::query_scalar!(
        r#"UPDATE products
       SET barcode = $1, name = $2, description = $3,category_id = $4,sub_category_id = $5, price = $6, is_active = $7,updated_by= $8, updated_at = NOW()
       WHERE id = $9
       RETURNING id
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category_id,
        payload.sub_category_id,
        payload.price,
        payload.is_active,
        payload.updated_by,
        payload.id,
    )
    .fetch_one(&mut **tx)
    .await.map_err(app_error::StoreError::Database)?;
    fetch_by_id(&mut **tx, product_id).await
}

pub async fn delete(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<(), app_error::StoreError> {
    let result = sqlx::query!(r#"DELETE from products WHERE id = $1"#, id)
        .execute(&mut **tx)
        .await?;
    let affected = result.rows_affected();

    if affected > 1 {
        return Err(app_error::StoreError::SafetyTrigger(affected));
    }
    if affected == 0 {
        return Err(app_error::StoreError::RecordNotFound);
    }
    Ok(())
}
