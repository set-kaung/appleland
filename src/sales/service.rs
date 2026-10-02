use super::{store as sale_store, store};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use crate::app_error::{APIError, StoreError};
use crate::inventory;
use crate::orders;
use crate::sales::model::{AdminSaleDetails, SaleActor, SaleProduct, SaleProductDTO};
use crate::users;

type InventoryDeduction = (Uuid, i32);
type SaleItemsWithDeductions = (Vec<SaleProductDTO>, Vec<InventoryDeduction>);

pub async fn compute_sale_total(
    pool: &PgPool,
    products: &[SaleProduct],
) -> Result<Decimal, StoreError> {
    let product_ids: Vec<Uuid> = products.iter().map(|p| p.product_id).collect();
    let prices = sale_store::fetch_selling_prices(pool, &product_ids).await?;

    let mut total = Decimal::ZERO;
    for p in products {
        let price = prices.get(&p.product_id).ok_or_else(|| {
            StoreError::InvalidInput(format!("product not found: {}", p.product_id))
        })?;
        total += *price * Decimal::from(p.quantity);
    }

    Ok(total)
}

pub async fn perform_sale_admin(
    pool: &PgPool,
    admin_id: Uuid,
    products: &[SaleProduct],
    payment_method: Option<&str>,
) -> Result<Uuid, StoreError> {
    perform_sale(
        pool,
        products,
        SaleActor::Admin(admin_id),
        None,
        None,
        payment_method,
        true,
    )
    .await
}

pub async fn perform_sale_user(
    pool: &PgPool,
    user_id: Uuid,
    products: &[SaleProduct],
    payment_order_id: &str,
    payment_provider: &str,
    payment_method: &str,
) -> Result<Uuid, StoreError> {
    tracing::debug!("perfrom_sale_user called");
    perform_sale(
        pool,
        products,
        SaleActor::User(user_id),
        Some(payment_order_id),
        Some(payment_provider),
        Some(payment_method),
        false,
    )
    .await
}

async fn perform_sale(
    pool: &PgPool,
    products: &[SaleProduct],
    actor: SaleActor,
    payment_order_id: Option<&str>,
    payment_provider: Option<&str>,
    payment_method: Option<&str>,
    is_instore: bool,
) -> Result<Uuid, StoreError> {
    let mut tx = pool.begin().await?;

    let product_ids: Vec<Uuid> = products.iter().map(|p| p.product_id).collect();

    let selling_prices: HashMap<Uuid, Decimal> =
        sale_store::fetch_selling_prices(&mut *tx, &product_ids).await?;

    let inventory_entries =
        inventory::store::fetch_inventory_fifo_batch(&mut *tx, &product_ids).await?;

    let (sale_product_dtos, deductions) =
        build_sale_items(products, &selling_prices, &inventory_entries)?;

    if let Err(err @ StoreError::UnexpectedRowsAffected(_)) =
        inventory::store::deduct_inventory_batch(&mut tx, &deductions).await
    {
        if let Ok(fresh_entries) =
            inventory::store::fetch_inventory_fifo_batch(pool, &product_ids).await
            && let Err(shortage @ StoreError::InsufficientInventory(..)) =
                build_sale_items(products, &selling_prices, &fresh_entries)
        {
            return Err(shortage);
        }
        return Err(err);
    }

    let sale_id = insert_sale_data(
        &mut tx,
        sale_product_dtos,
        is_instore,
        payment_order_id,
        payment_provider,
        payment_method,
    )
    .await?;

    match actor {
        SaleActor::Admin(admin_id) => {
            sale_store::insert_admin_sale(&mut tx, sale_id, admin_id).await?;
        }
        SaleActor::User(user_id) => {
            sale_store::insert_user_sale(&mut tx, sale_id, user_id).await?;
            insert_user_order(&mut tx, sale_id, user_id).await?;
        }
    }

    tx.commit().await?;

    Ok(sale_id)
}

// copies the user's delivery contact into a new delivery_contact row,
// then attaches an order to the sale. fails the sale if the user has no
// saved contact.
async fn insert_user_order(
    tx: &mut Transaction<'_, Postgres>,
    sale_id: Uuid,
    user_id: Uuid,
) -> Result<(), StoreError> {
    let user_contact = users::store::fetch_user_contact_by_user_id(&mut **tx, user_id)
        .await?
        .ok_or_else(|| {
            StoreError::InvalidInput("user has no saved delivery contact".to_string())
        })?;

    orders::service::insert_user_order(
        tx,
        sale_id,
        user_id,
        user_contact.address_line_1,
        user_contact.phone_number,
    )
    .await
}

pub fn validate_aggregate_sale_products(
    products: &[SaleProduct],
) -> Result<Vec<SaleProduct>, APIError> {
    if products.is_empty() {
        return Err(APIError::BadRequest(
            "sale must contain at least one product".to_string(),
        ));
    }

    let mut aggregated: HashMap<Uuid, i32> = HashMap::new();

    for product in products {
        if product.quantity <= 0 {
            return Err(APIError::BadRequest(
                "quantity must be greater than 0".to_string(),
            ));
        }

        let total = aggregated.entry(product.product_id).or_insert(0);
        *total = total
            .checked_add(product.quantity)
            .ok_or_else(|| APIError::BadRequest("quantity overflow".to_string()))?;
    }

    Ok(aggregated
        .into_iter()
        .map(|(product_id, quantity)| SaleProduct {
            product_id,
            quantity,
        })
        .collect())
}

// check if there is enough inventory,
// create the SaleProductDTO Vector (with inventory ids) for insert_sale_data
pub(super) fn build_sale_items(
    sale_products: &[SaleProduct],
    selling_prices: &HashMap<Uuid, Decimal>,
    inventory_entries: &[inventory::model::InventoryRow],
) -> Result<SaleItemsWithDeductions, StoreError> {
    let mut inventory_by_product: HashMap<Uuid, Vec<&inventory::model::InventoryRow>> =
        HashMap::new();

    // create inventory HashMap for deduction lookup
    for entry in inventory_entries {
        inventory_by_product
            .entry(entry.product_id)
            .or_default()
            .push(entry);
    }

    // DTO data for insert_sale_data
    let mut items: Vec<SaleProductDTO> = Vec::new();

    // track which inventory record need to be deducted by how much
    let mut deductions: Vec<(Uuid, i32)> = Vec::new();

    for product in sale_products {
        let entries = inventory_by_product
            .get(&product.product_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        // check if there is available inventories for the sale
        let total_available: i32 = entries.iter().map(|e| e.quantity_remaining).sum();
        if total_available < product.quantity {
            return Err(StoreError::InsufficientInventory(
                product.product_id,
                product.quantity,
                total_available,
            ));
        }

        let selling_price = *selling_prices
            .get(&product.product_id)
            .ok_or(StoreError::RecordNotFound)?;

        // track how much items still need to deduct from product inventory
        let mut remaining_sale_quantity = product.quantity;

        for entry in entries {
            if remaining_sale_quantity <= 0 {
                break;
            }
            let deduct = remaining_sale_quantity.min(entry.quantity_remaining);

            deductions.push((entry.id, deduct));

            items.push(SaleProductDTO {
                product_id: product.product_id,
                quantity: deduct,
                cost_price: entry.unit_cost,
                inventory_id: entry.id,
                selling_price,
            });

            remaining_sale_quantity -= deduct;
        }
    }

    Ok((items, deductions))
}

// this function assume the inventory checks are done
pub async fn insert_sale_data(
    tx: &mut Transaction<'_, Postgres>,
    sale_products: Vec<SaleProductDTO>,
    is_instore: bool,
    payment_order_id: Option<&str>,
    payment_provider: Option<&str>,
    payment_method: Option<&str>,
) -> Result<Uuid, StoreError> {
    let total_selling_price: Decimal = sale_products
        .iter()
        .map(|x| x.selling_price * Decimal::from(x.quantity))
        .sum();

    let sale_id = sale_store::insert_sale(
        tx,
        total_selling_price,
        is_instore,
        payment_order_id,
        payment_provider,
        payment_method,
    )
    .await?;

    sale_store::insert_sale_items(tx, sale_id, sale_products).await?;

    Ok(sale_id)
}

pub async fn get_admin_sale_detail_by_id(
    pool: &PgPool,
    sale_id: Uuid,
) -> Result<AdminSaleDetails, APIError> {
    let (sale_row, sale_items, sale_admin) = tokio::try_join!(
        store::fetch_sale_row_by_id(pool, sale_id),
        store::fetch_sale_items(pool, sale_id),
        store::fetch_sale_admin_by_sale_id(pool, sale_id),
    )?;
    Ok(AdminSaleDetails {
        id: sale_row.id,
        total_amount: sale_row.total_amount,
        created_at: sale_row.created_at,
        is_instore_sale: sale_row.is_instore_sale,
        sale_items,
        by_admin: sale_admin,
    })
}
