use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::admin_settings::model::{SettingResponse, SettingValueType};
use crate::admin_settings::store;
use crate::app_error::{APIError, log_err_convert};

pub async fn get_all_settings_service(pool: &PgPool) -> Result<Vec<SettingResponse>, APIError> {
    let rows = store::get_all_settings(pool).await?;
    Ok(rows.into_iter().map(SettingResponse::from).collect())
}

pub async fn update_setting_value_service(
    pool: &PgPool,
    key: &str,
    value: serde_json::Value,
    admin_id: Uuid,
) -> Result<SettingResponse, APIError> {
    let existing = store::get_setting_by_key(pool, key)
        .await?
        .ok_or(APIError::NotFound)?;

    let value_type = SettingValueType::try_from(existing.value_type.as_str()).map_err(|e| {
        log_err_convert(
            "failed to parse setting value type",
            e,
            APIError::InternalErr,
        )
    })?;

    if !value_type.is_valid(&value) {
        return Err(APIError::BadRequest(format!(
            "value for setting '{key}' must be of type {value_type}"
        )));
    }

    let updated = store::update_setting_value(pool, key, value, Some(admin_id)).await?;
    Ok(updated.into())
}

pub async fn get_setting_by_key_service<'e, E>(
    executor: E,
    key: &str,
) -> Result<SettingResponse, APIError>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = store::get_setting_by_key(executor, key).await?;
    match result {
        Some(v) => Ok(v.into()),
        None => Err(APIError::BadRequest(format! {"no {key} setting"})),
    }
}
