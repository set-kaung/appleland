use crate::admin_settings::model::{NewSetting, SettingRow};
use crate::app_error::StoreError;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

pub async fn get_all_settings(pool: &PgPool) -> Result<Vec<SettingRow>, StoreError> {
    let rows = sqlx::query_as!(
        SettingRow,
        r#"
        SELECT key, value, description, type AS value_type, created_at, updated_at, updated_by
        FROM settings
        ORDER BY key ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[allow(dead_code)]
pub async fn get_setting_by_key<'e, E>(
    executor: E,
    key: &str,
) -> Result<Option<SettingRow>, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let row = sqlx::query_as!(
        SettingRow,
        r#"
        SELECT key, value, description, type AS value_type, created_at, updated_at, updated_by
        FROM settings
        WHERE key = $1
        "#,
        key
    )
    .fetch_optional(executor)
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
        INSERT INTO settings (key, value, description, type, updated_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING key, value, description, type AS value_type, created_at, updated_at, updated_by
        "#,
        new_setting.key,
        new_setting.value,
        new_setting.description,
        new_setting.value_type.as_str(),
        new_setting.updated_by,
    )
    .fetch_one(executor)
    .await?;

    Ok(result)
}

pub async fn update_setting_value<'e, E>(
    executor: E,
    key: &str,
    value: serde_json::Value,
    updated_by: Option<Uuid>,
) -> Result<SettingRow, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query_as!(
        SettingRow,
        r#"
        UPDATE settings
        SET value = $2, updated_by = $3, updated_at = NOW()
        WHERE key = $1
        RETURNING key, value, description, type AS value_type, created_at, updated_at, updated_by
        "#,
        key,
        value,
        updated_by,
    )
    .fetch_one(executor)
    .await?;

    Ok(result)
}
