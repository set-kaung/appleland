use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::model::DeliveryStatus;
// use crate::{app_error::StoreError, orders::model::InsertOrder};
use crate::app_error::StoreError;

// pub async fn insert_order(
//     tx: &mut Transaction<'_, Postgres>,
//     insert_order: InsertOrder,
// ) -> Result<Uuid, StoreError> {
//     let result: Uuid = sqlx::query_scalar!(
//         "INSERT INTO orders (sale_id, shipping_address_id, order_method,created_by, updated_by)
//          VALUES($1,$2,$3,$4,$5)
//          RETURNING id",
//         insert_order.sale_id,
//         insert_order.shipping_address_id,
//         insert_order.order_method,
//         insert_order.created_by,
//         insert_order.updated_by
//     )
//     .fetch_one(&mut **tx)
//     .await?;
//     Ok(result)
// }

pub async fn update_order_tracking_url(
    pool: &PgPool,
    order_id: Uuid,
    tracking_url: String,
) -> Result<Uuid, StoreError> {
    let result = sqlx::query_scalar!(
        "UPDATE orders SET order_tracking_url = $1, delivery_status = $2 WHERE id = $3 RETURNING id",
        tracking_url,
        String::from(&DeliveryStatus::OnDelivery),
        order_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result)
}

pub async fn get_order_delivery_status<'e, E>(
    executor: E,
    order_id: Uuid,
) -> Result<String, StoreError>
where
    E: Executor<'e, Database = Postgres>,
{
    let result: String =
        sqlx::query_scalar!("SELECT delivery_status FROM orders WHERE id = $1", order_id)
            .fetch_one(executor)
            .await?;

    Ok(result)
}

pub async fn update_order_delivery_status(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    delivery_status: String,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE orders SET delivery_status = $1 WHERE id = $2",
        delivery_status,
        id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// pub async fn get_all_orders_paged(pool: &PgPool, limit: i64,offset: i64)
