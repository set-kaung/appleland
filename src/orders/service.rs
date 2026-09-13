use sqlx::PgPool;
use uuid::Uuid;

use super::store;
use crate::{
    app_error::{APIError, log_err_convert},
    orders::model::{DeliveryStatus, UpdateOrderDeliveryStatusResponse},
};

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
