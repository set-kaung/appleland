use crate::admin_settings::model::{NewSetting, SettingRow};
use crate::app_error::StoreError;
use sqlx::{Executor, PgPool, Postgres};

#[allow(dead_code)]
pub async fn get_all_settings(pool: &PgPool) -> Result<Vec<SettingRow>, StoreError> {
    let rows = sqlx::query_as!(
        SettingRow,
        r#"
        SELECT key, value, description, type, created_at, updated_at, created_by, updated_by
        FROM settings
        ORDER BY key ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[allow(dead_code)]
pub async fn get_setting_by_key(
    pool: &PgPool,
    key: &String,
) -> Result<Option<SettingRow>, StoreError> {
    let row = sqlx::query_as!(
        SettingRow,
        r#"
        SELECT key, value, description, type, created_at, updated_at, created_by, updated_by
        FROM settings
        WHERE key = $1
        "#,
        key
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

#[allow(dead_code)]
pub async fn insert_setting<'e, E>(
    executor: E,
    new_setting: NewSetting,
) -> Result<SettingRow, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query_as!(
        SettingRow,
        r#"
        INSERT INTO settings (key, value, description, type, created_by, updated_by)
        VALUES ($1, $2, $3, $4, $5, $5)
        RETURNING key, value, description, type, created_at, updated_at, created_by, updated_by
        "#,
        new_setting.key,
        new_setting.value,
        new_setting.description,
        new_setting.r#type.as_str(),
        new_setting.created_by,
    )
    .fetch_one(executor)
    .await?;

    Ok(result)
}
