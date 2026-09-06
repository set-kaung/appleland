use crate::app_error::StoreError;
use crate::category::model::{CategoryRow, DeleteCategoryResponse, InsertCategory, UpdateCategory};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn create_category(
    tx: &mut Transaction<'_, Postgres>,
    insert_category: InsertCategory,
) -> Result<CategoryRow, StoreError> {
    let category_row = sqlx::query_as!(
        CategoryRow,
        r#"
            INSERT INTO categories (name,created_by,updated_by,created_at,updated_at)
            VALUES ($1,$2,$3,NOW(),NOW())
            RETURNING id,name,created_by,created_at,updated_by,updated_at,is_deleted, deleted_at as "deleted_at?", deleted_by as "deleted_by?"
        "#,
        insert_category.name,
        insert_category.created_by,
        insert_category.updated_by
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(category_row)
}
pub async fn get_all_categories(pool: &PgPool) -> Result<Vec<CategoryRow>, StoreError> {
    sqlx::query_as!(
        CategoryRow,
        r#"SELECT id, name, created_by, created_at, updated_by, updated_at,is_deleted, deleted_at as "deleted_at?", deleted_by as "deleted_by?" FROM categories"#
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn update_category(
    pool: &PgPool,
    update_category: UpdateCategory,
) -> Result<CategoryRow, StoreError> {
    let result = sqlx::query_as!(
        CategoryRow,
        r#"UPDATE categories SET name = $1, updated_by = $2 WHERE id = $3 RETURNING id, name, created_by, created_at, updated_by, updated_at,is_deleted, deleted_at as "deleted_at?", deleted_by as "deleted_by?""#,
        update_category.name,
        update_category.updated_by,
        update_category.id
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn delete_category(
    pool: &PgPool,
    admin_id: Uuid,
    category_id: Uuid,
) -> Result<DeleteCategoryResponse, StoreError> {
    let result = sqlx::query_as!(
        DeleteCategoryResponse,
        r#"
            UPDATE categories SET
                is_deleted = $1,
                deleted_at = NOW(),
                deleted_by = $2
                WHERE id = $3
            RETURNING id, is_deleted, deleted_at, deleted_by;
    "#,
        true,
        admin_id,
        category_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result)
}
