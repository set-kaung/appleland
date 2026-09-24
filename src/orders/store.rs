use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use super::model::{AdminOrderRow, AdminOrdersQuery, DeliveryStatus, InsertOrder};
use crate::app_error::StoreError;
use crate::db::DBQueryFilter;

pub async fn get_all_orders_paginated(
    pool: &PgPool,
    filter: &AdminOrdersQuery,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdminOrderRow>, i64), StoreError> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            o.id,
            o.sale_id,
            COALESCE(u.username, a.username) AS customer,
            s.total_amount,
            o.delivery_status,
            o.order_tracking_url,
            o.created_at
        FROM orders o
        JOIN sales s ON s.id = o.sale_id
        LEFT JOIN user_sales us ON us.sale_id = o.sale_id
        LEFT JOIN users u ON u.id = us.user_id
        LEFT JOIN admin_sales ads ON ads.sale_id = o.sale_id
        LEFT JOIN admins a ON a.id = ads.admin_id
        WHERE 1 = 1
        "#,
    );
    filter.apply_filter(&mut query_builder);
    query_builder.push(" ORDER BY o.created_at DESC, o.id DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let rows = query_builder
        .build_query_as::<AdminOrderRow>()
        .fetch_all(pool)
        .await?;

    let mut count_builder = QueryBuilder::new(
        r#"
        SELECT COUNT(*) FROM orders o
        WHERE 1 = 1
        "#,
    );
    filter.apply_filter(&mut count_builder);

    let total_items = count_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;

    Ok((rows, total_items))
}

pub async fn insert_delivery_contact(
    tx: &mut Transaction<'_, Postgres>,
    address_line_1: String,
    phone_number: String,
) -> Result<Uuid, StoreError> {
    let result: Uuid = sqlx::query_scalar!(
        "INSERT INTO delivery_contact (address_line_1, phone_number)
         VALUES($1,$2)
         RETURNING id",
        address_line_1,
        phone_number
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(result)
}

pub async fn insert_order(
    tx: &mut Transaction<'_, Postgres>,
    insert_order: InsertOrder,
) -> Result<Uuid, StoreError> {
    let result: Uuid = sqlx::query_scalar!(
        "INSERT INTO orders (sale_id, shipping_address_id, order_method,created_by, updated_by)
         VALUES($1,$2,$3,$4,$5)
         RETURNING id",
        insert_order.sale_id,
        insert_order.shipping_address_id,
        insert_order.order_method,
        insert_order.created_by,
        insert_order.updated_by
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(result)
}

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
