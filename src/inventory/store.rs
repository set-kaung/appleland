use uuid::Uuid;

use super::model as inv_model;
use crate::app_error;

pub async fn insert_inventory_row(
    product_id: Uuid,
    admin_id: Uuid,
    quantity: i64,
) -> Result<(), app_error::StoreError> {
    sqlx::query!(
        r#"insert into inventory (product_id,quantity,created_by,updated_by)
    VALUES($1,$2,$3,$3)
    "#,
        product_id,
        quantity,
        admin_id,
        admin_id
    )
}
