use crate::app_error::StoreError;
use crate::category::model::{CategoryRow, InsertCategory};
use sqlx::{PgPool, Postgres, Transaction};

pub async fn create_category(
    tx: &mut Transaction<'_, Postgres>,
    insert_category: InsertCategory,
) -> Result<CategoryRow, StoreError> {
    let category_row = sqlx::query_as!(
        CategoryRow,
        r#"
            INSERT INTO categories (name,created_by,updated_by,created_at,updated_at)
            VALUES ($1,$2,$3,NOW(),NOW())
            RETURNING id,name,created_by,created_at,updated_by,updated_at
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
    sqlx::query_as!(CategoryRow, "SELECT * FROM categories")
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}
