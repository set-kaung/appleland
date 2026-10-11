use sqlx::{Executor, PgPool, Postgres};

use crate::admin_settings::model::SettingResponse;
use crate::admin_settings::store;
use crate::app_error::APIError;

pub async fn get_all_settings_service(pool: &PgPool) -> Result<Vec<SettingResponse>, APIError> {
    let rows = store::get_all_settings(pool).await?;
    Ok(rows.into_iter().map(SettingResponse::from).collect())
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
