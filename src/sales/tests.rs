#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::app_error::StoreError;
use crate::inventory::model::InventoryRow;
use crate::sales::model::SaleProduct;
use crate::sales::service::build_sale_items;
use rust_decimal::Decimal;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

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
