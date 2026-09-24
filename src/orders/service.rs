use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::model::InsertOrder;
use super::store;
use crate::{
    app_error::{APIError, StoreError, log_err_convert},
    orders::model::{DeliveryStatus, UpdateOrderDeliveryStatusResponse},
};

pub async fn insert_user_order(
    tx: &mut Transaction<'_, Postgres>,
    sale_id: Uuid,
    user_id: Uuid,
    address_line_1: String,
    phone_number: String,
) -> Result<(), StoreError> {
    let shipping_address_id =
        store::insert_delivery_contact(tx, address_line_1, phone_number).await?;

    store::insert_order(
        tx,
        InsertOrder {
            sale_id,
            shipping_address_id,
            order_method: String::from("app order"),
            created_by: user_id,
            updated_by: user_id,
        },
    )
    .await?;

    Ok(())
}

pub async fn update_order_delivery_status(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<super::model::UpdateOrderDeliveryStatusResponse, APIError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| log_err_convert("failed to start transaction", e, APIError::InternalErr))?;
    let delivery_status = store::get_order_delivery_status(&mut *tx, order_id).await?;
    let delivery_status_enum = DeliveryStatus::try_from(delivery_status.as_str()).map_err(|e| {
        log_err_convert(
            "failed to convert delivery status enum",
            e,
            APIError::InternalErr,
        )
    })?;

    let next_status = delivery_status_enum
        .next()
        .map_err(|e| log_err_convert("failed to convert enum", e, APIError::InternalErr))?;

    store::update_order_delivery_status(&mut tx, order_id, String::from(&next_status)).await?;

    if let Err(e) = tx.commit().await {
        return Err(log_err_convert(
            "failed to commit transaction",
            e,
            APIError::InternalErr,
        ));
    }
    Ok(UpdateOrderDeliveryStatusResponse {
        id: order_id,
        delivery_status: next_status,
    })
}
