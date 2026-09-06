use super::store as sale_store;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use crate::app_error::{APIError, StoreError};
use crate::inventory;
use crate::sales::model::{SaleProduct, SaleProductDTO};

type InventoryDeduction = (Uuid, i32);
type SaleItemsWithDeductions = (Vec<SaleProductDTO>, Vec<InventoryDeduction>);

pub async fn perform_sale_admin(
    pool: &PgPool,
    admin_id: Uuid,
    products: &[SaleProduct],
) -> Result<Uuid, StoreError> {
    let mut tx = pool.begin().await?;

    let product_ids: Vec<Uuid> = products.iter().map(|p| p.product_id).collect();

    let selling_prices: HashMap<Uuid, Decimal> =
        sale_store::fetch_selling_prices(&mut tx, &product_ids).await?;

    let inventory_entries =
        inventory::store::fetch_inventory_fifo_batch(&mut *tx, &product_ids).await?;

    let (sale_product_dtos, deductions) =
        build_sale_items(products, &selling_prices, &inventory_entries)?;

    if let Err(err @ StoreError::UnexpectedRowsAffected(_)) =
        inventory::store::deduct_inventory_batch(&mut tx, &deductions).await
    {
        // stock was validated against this snapshot, so a short UPDATE means a
        // concurrent sale deducted first; re-check with a fresh snapshot (on the
        // pool, so our rolled-back partial deductions are not visible) to report
        // which product is short
        if let Ok(fresh_entries) =
            inventory::store::fetch_inventory_fifo_batch(pool, &product_ids).await
            && let Err(shortage @ StoreError::InsufficientInventory(..)) =
                build_sale_items(products, &selling_prices, &fresh_entries)
            {
                return Err(shortage);
            }
        return Err(err);
    }

    let sale_id = insert_sale_data(&mut tx, sale_product_dtos).await?;
    sale_store::insert_admin_sale(&mut tx, sale_id, admin_id).await?;

    tx.commit().await?;

    Ok(sale_id)
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
fn build_sale_items(
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
) -> Result<Uuid, StoreError> {
    let total_selling_price: Decimal = sale_products
        .iter()
        .map(|x| x.selling_price * Decimal::from(x.quantity))
        .sum();

    let sale_id = sale_store::insert_sale(tx, total_selling_price).await?;

    sale_store::insert_sale_items(tx, sale_id, sale_products).await?;

    Ok(sale_id)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::inventory::model::InventoryRow;
    use time::OffsetDateTime;

    fn make_inventory(product_id: Uuid, quantity_remaining: i32, unit_cost: i64) -> InventoryRow {
        InventoryRow {
            id: Uuid::new_v4(),
            product_id,
            quantity_received: quantity_remaining,
            quantity_remaining,
            unit_cost: Decimal::new(unit_cost, 2),
            created_by: Uuid::new_v4(),
            updated_by: Uuid::new_v4(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            received_at: OffsetDateTime::now_utc(),
            expiry_date: Some(OffsetDateTime::now_utc()),
        }
    }

    fn make_sale_product(product_id: Uuid, quantity: i32) -> SaleProduct {
        SaleProduct {
            product_id,
            quantity,
        }
    }

    #[test]
    fn test_enough_inventory_single_entry() {
        let product_id = Uuid::new_v4();
        let selling_prices = HashMap::from([(product_id, Decimal::new(500, 2))]);
        let inventory = vec![make_inventory(product_id, 10, 100)];
        let sale_products = vec![make_sale_product(product_id, 5)];

        let (items, deductions) =
            build_sale_items(&sale_products, &selling_prices, &inventory).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].quantity, 5);
        assert_eq!(items[0].cost_price, Decimal::new(100, 2));
        assert_eq!(items[0].selling_price, Decimal::new(500, 2));
        assert_eq!(items[0].inventory_id, inventory[0].id);
        assert_eq!(deductions, vec![(inventory[0].id, 5)]);
    }

    #[test]
    fn test_enough_inventory_fifo_split_across_entries() {
        let product_id = Uuid::new_v4();
        let selling_prices = HashMap::from([(product_id, Decimal::new(500, 2))]);
        let inv1 = make_inventory(product_id, 3, 100);
        let inv2 = make_inventory(product_id, 7, 150);
        let inventory = vec![inv1.clone(), inv2.clone()];
        let sale_products = vec![make_sale_product(product_id, 8)];

        let (items, deductions) =
            build_sale_items(&sale_products, &selling_prices, &inventory).unwrap();

        assert_eq!(items.len(), 2);

        assert_eq!(items[0].quantity, 3);
        assert_eq!(items[0].cost_price, Decimal::new(100, 2));
        assert_eq!(items[0].inventory_id, inv1.id);

        assert_eq!(items[1].quantity, 5);
        assert_eq!(items[1].cost_price, Decimal::new(150, 2));
        assert_eq!(items[1].inventory_id, inv2.id);

        assert_eq!(deductions, vec![(inv1.id, 3), (inv2.id, 5)]);
    }

    #[test]
    fn test_not_enough_inventory() {
        let product_id = Uuid::new_v4();
        let selling_prices = HashMap::from([(product_id, Decimal::new(500, 2))]);
        let inventory = vec![make_inventory(product_id, 3, 100)];
        let sale_products = vec![make_sale_product(product_id, 5)];

        let result = build_sale_items(&sale_products, &selling_prices, &inventory);

        assert!(matches!(
            result,
            Err(StoreError::InsufficientInventory(id, 5, 3)) if id == product_id
        ));
    }

    #[test]
    fn test_zero_inventory_entries() {
        let product_id = Uuid::new_v4();
        let selling_prices = HashMap::from([(product_id, Decimal::new(500, 2))]);
        let inventory: Vec<InventoryRow> = vec![];
        let sale_products = vec![make_sale_product(product_id, 1)];

        let result = build_sale_items(&sale_products, &selling_prices, &inventory);

        assert!(matches!(
            result,
            Err(StoreError::InsufficientInventory(id, 1, 0)) if id == product_id
        ));
    }

    #[test]
    fn test_multiple_products() {
        let product_a = Uuid::new_v4();
        let product_b = Uuid::new_v4();
        let selling_prices = HashMap::from([
            (product_a, Decimal::new(500, 2)),
            (product_b, Decimal::new(800, 2)),
        ]);
        let inv_a = make_inventory(product_a, 10, 100);
        let inv_b = make_inventory(product_b, 5, 200);
        let inventory = vec![inv_a.clone(), inv_b.clone()];
        let sale_products = vec![
            make_sale_product(product_a, 4),
            make_sale_product(product_b, 3),
        ];

        let (items, deductions) =
            build_sale_items(&sale_products, &selling_prices, &inventory).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].product_id, product_a);
        assert_eq!(items[0].quantity, 4);
        assert_eq!(items[1].product_id, product_b);
        assert_eq!(items[1].quantity, 3);

        assert_eq!(deductions, vec![(inv_a.id, 4), (inv_b.id, 3)]);
    }
}
