use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::model::{
    AdminOrderDetail, InsertOrder, OrderDetail, OrderDetailRow, OrderItemRow,
    OrderShippingAddress, OrderStatusHistoryRow, UserOrderDetail,
};
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

    let order_id = store::insert_order(
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

    store::insert_order_status_history(
        tx,
        order_id,
        String::from(&DeliveryStatus::Pending),
        Some(user_id),
    )
    .await?;

    Ok(())
}

pub async fn update_order_delivery_status(
    pool: &PgPool,
    order_id: Uuid,
    changed_by: Option<Uuid>,
) -> Result<UpdateOrderDeliveryStatusResponse, APIError> {
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
        .map_err(|_| APIError::Conflict("can't advance past received state".to_string()))?;

    store::update_order_delivery_status(&mut tx, order_id, String::from(&next_status)).await?;
    store::insert_order_status_history(&mut tx, order_id, String::from(&next_status), changed_by)
        .await?;

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

pub async fn update_order_tracking_url(
    pool: &PgPool,
    order_id: Uuid,
    tracking_url: String,
    changed_by: Option<Uuid>,
) -> Result<Uuid, APIError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| log_err_convert("failed to start transaction", e, APIError::InternalErr))?;

    let order_id = store::update_order_tracking_url(&mut *tx, order_id, tracking_url).await?;
    store::insert_order_status_history(
        &mut tx,
        order_id,
        String::from(&DeliveryStatus::OnDelivery),
        changed_by,
    )
    .await?;

    if let Err(e) = tx.commit().await {
        return Err(log_err_convert(
            "failed to commit transaction",
            e,
            APIError::InternalErr,
        ));
    }
    Ok(order_id)
}

fn assemble_order_detail(
    detail_row: OrderDetailRow,
    item_rows: Vec<OrderItemRow>,
    status_rows: Vec<OrderStatusHistoryRow>,
) -> OrderDetail {
    let items = item_rows.into_iter().map(Into::into).collect();
    let status_history = status_rows.into_iter().map(Into::into).collect();

    OrderDetail {
        id: detail_row.id,
        sale_id: detail_row.sale_id,
        order_method: detail_row.order_method,
        delivery_status: detail_row.delivery_status,
        order_tracking_url: detail_row.order_tracking_url,
        customer: detail_row.customer,
        total_amount: detail_row.total_amount,
        is_instore_sale: detail_row.is_instore_sale,
        payment_method: detail_row.payment_method,
        payment_provider: detail_row.payment_provider,
        payment_order_id: detail_row.payment_order_id,
        shipping_address: OrderShippingAddress {
            address_line_1: detail_row.shipping_address_line_1,
            phone_number: detail_row.shipping_address_phone_number,
        },
        items,
        status_history,
        created_at: detail_row.created_at,
    }
}

pub async fn get_order_detail(pool: &PgPool, order_id: Uuid) -> Result<OrderDetail, APIError> {
    let (detail_row, item_rows, status_rows) = tokio::try_join!(
        store::fetch_order_detail(pool, order_id),
        store::fetch_order_items(pool, order_id),
        store::fetch_order_status_history(pool, order_id)
    )?;

    Ok(assemble_order_detail(detail_row, item_rows, status_rows))
}

pub async fn get_admin_order_detail(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<AdminOrderDetail, APIError> {
    let detail = get_order_detail(pool, order_id).await?;
    Ok(detail.into())
}

pub async fn get_user_order_detail(
    pool: &PgPool,
    order_id: Uuid,
    user_id: Uuid,
) -> Result<UserOrderDetail, APIError> {
    let detail_row = store::fetch_order_detail_for_user(pool, order_id, user_id)
        .await?
        .ok_or(APIError::NotFound)?;

    let (item_rows, status_rows) = tokio::try_join!(
        store::fetch_order_items(pool, order_id),
        store::fetch_order_status_history(pool, order_id)
    )?;

    let detail = assemble_order_detail(detail_row, item_rows, status_rows);
    Ok(detail.into())
}
