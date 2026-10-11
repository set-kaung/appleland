use sqlx::{Executor, Postgres};

use crate::admin_settings::model::SettingResponse;
use crate::admin_settings::store;
use crate::app_error::APIError;

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
