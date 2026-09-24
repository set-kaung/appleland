use super::model::CreateUserPayload;
use super::store;
use crate::app_error::{APIError, StoreError, log_err_convert};
use sqlx::PgPool;
use uuid::Uuid;

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

    tx.commit()
        .await
        .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;

    Ok(user_id)
}
