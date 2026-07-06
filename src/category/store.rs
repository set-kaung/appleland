use sqlx::{Postgres, Transaction};
use crate::app_error::StoreError;
use crate::category::model::{CategoryRow, InsertCategory};

pub async fn create_category(tx: &mut Transaction<'_, Postgres>, insert_category: InsertCategory) -> Result<CategoryRow,StoreError>{
    let category_row = sqlx::query_as!(CategoryRow,r#"
            INSERT INTO categories (name,created_by,updated_by,created_at,updated_at)
            VALUES ($1,$2,$3,NOW(),NOW())
            RETURNING id,name,created_by,created_at,updated_by,updated_at
        "#,
        insert_category.name,
        insert_category.created_by,
        insert_category.updated_by
    ).fetch_one(&mut **tx).await?;
    Ok(category_row)
}