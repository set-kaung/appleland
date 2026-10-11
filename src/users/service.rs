use super::model::CreateUserPayload;
use super::store;
use crate::{
    admin_settings,
    app_error::{APIError, StoreError, log_err_convert},
    users::model::UserPointsBalanceUpdateResult,
};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use admin_settings::model::SettingValueType;

use crate::playground::service as playground_service;

pub async fn find_user_id_by_supabase_id(
    pool: &PgPool,
    supabase_id: Uuid,
) -> Result<Option<Uuid>, StoreError> {
    store::find_user_id_by_supabase_id(pool, supabase_id).await
}

pub async fn new_user_account_creation(
    pool: &PgPool,
    supabase_id: Uuid,
    user_creation_payload: CreateUserPayload,
) -> Result<Uuid, APIError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| log_err_convert("cannot begin transaction", e, APIError::InternalErr))?;

    // link application user id with supabase id
    let user_id =
        store::insert_into_users_supabase(&mut tx, supabase_id, user_creation_payload.username)
            .await?;

    // insert user contact
    store::insert_into_user_contact(
        &mut tx,
        user_id,
        user_creation_payload.address_line_1,
        user_creation_payload.phone_number,
    )
    .await?;

    // create playground user
    playground_service::create_playground_account(&mut tx, user_id).await?;

    store::insert_user_store_points(&mut tx, user_id).await?;

    tx.commit()
        .await
        .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;

    Ok(user_id)
}

pub async fn add_user_points_from_sale(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    sale_id: Uuid,
    kyats: Decimal,
) -> Result<UserPointsBalanceUpdateResult, APIError> {
    let setting =
        admin_settings::service::get_setting_by_key_service(&mut **tx, "kyat_per_point").await?;
    let setting_type = SettingValueType::try_from(setting.value_type.as_str()).map_err(|e| {
        log_err_convert(
            "failed to convert setting value type",
            e,
            APIError::InternalErr,
        )
    })?;
    if matches!(setting_type, SettingValueType::Decimal) {
        let kyat_to_point: Decimal = serde_json::from_value(setting.value)
            .map_err(|e| log_err_convert("invalid decimal setting", e, APIError::InternalErr))?;
        if kyat_to_point == Decimal::ZERO {
            Err(APIError::InternalErr)
        } else {
            let raw = kyats.checked_div(kyat_to_point).ok_or_else(|| {
                tracing::error!(
                    kyats = %kyats,
                    kyat_to_point = %kyat_to_point,
                    "checked_div returned None (divisor was zero)"
                );
                APIError::InternalErr
            })?;

            let rounded = raw.round();

            let points_to_add = rounded.to_i32().ok_or_else(|| {
                tracing::error!(
                    kyats = %kyats,
                    kyat_to_point = %kyat_to_point,
                    raw = %raw,
                    rounded = %rounded,
                    "rounded points out of i32 range"
                );
                APIError::InternalErr
            })?;
            let final_balance =
                store::update_user_store_points_balance(tx, user_id, points_to_add).await?;

            if points_to_add != 0 {
                store::insert_user_point_transaction(tx, user_id, sale_id, points_to_add, "earn")
                    .await?;
            }

            Ok(UserPointsBalanceUpdateResult {
                id: user_id,
                points_added: points_to_add,
                final_balance,
            })
        }
    } else {
        tracing::error!(
            "setting value does not match expected type, expected {} vs actual: {}",
            SettingValueType::Decimal,
            setting_type
        );
        Err(APIError::InternalErr)
    }
}
