use sqlx::PgPool;

use crate::admin_settings::model::SettingResponse;
use crate::admin_settings::store;
use crate::app_error::APIError;

pub async fn get_setting_by_key_service(
    pool: &PgPool,
    key: String,
) -> Result<SettingResponse, APIError> {
    let result = store::get_setting_by_key(pool, &key).await?;
    match result {
        Some(v) => Ok(v.into()),
        None => Err(APIError::BadRequest(format! {"no {key} setting"})),
    }
}
