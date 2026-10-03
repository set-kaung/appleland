#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::app_error::StoreError;
use crate::inventory::model::{InventoryBatchInfo, InventoryBatchRow};
use crate::inventory::service::build_fifo_deductions;
use time::OffsetDateTime;
use uuid::Uuid;

fn make_batch(
    product_id: Uuid,
    quantity_received: i32,
    quantity_remaining: i32,
) -> InventoryBatchRow {
    InventoryBatchRow {
        id: Uuid::new_v4(),
        product_id,
        quantity_received,
        quantity_remaining,
        received_at: OffsetDateTime::now_utc(),
    }
}

#[test]
fn unknown_batch_writeoff_takes_oldest_stock_first() {
    let product_id = Uuid::new_v4();
    let oldest = make_batch(product_id, 10, 10);
    let newest = make_batch(product_id, 10, 10);
    let oldest_id = oldest.id;

    let deductions = build_fifo_deductions(product_id, &[oldest, newest], 8).unwrap();

    assert_eq!(deductions, vec![(oldest_id, 8)]);
}

#[test]
fn unknown_batch_writeoff_spans_batches_in_received_order() {
    let product_id = Uuid::new_v4();
    let first = make_batch(product_id, 5, 2);
    let second = make_batch(product_id, 8, 3);
    let third = make_batch(product_id, 20, 10);
    let first_id = first.id;
    let second_id = second.id;
    let third_id = third.id;

    let deductions = build_fifo_deductions(product_id, &[first, second, third], 6).unwrap();

    assert_eq!(
        deductions,
        vec![(first_id, 2), (second_id, 3), (third_id, 1)]
    );
}

#[test]
fn full_stock_writeoff_drains_every_batch() {
    let product_id = Uuid::new_v4();
    let first = make_batch(product_id, 5, 5);
    let second = make_batch(product_id, 7, 2);
    let first_id = first.id;
    let second_id = second.id;

    let deductions = build_fifo_deductions(product_id, &[first, second], 7).unwrap();

    assert_eq!(deductions, vec![(first_id, 5), (second_id, 2)]);
}

#[test]
fn writeoff_larger_than_stock_reports_requested_and_available() {
    let product_id = Uuid::new_v4();
    let batches = [make_batch(product_id, 5, 3), make_batch(product_id, 8, 4)];

    let result = build_fifo_deductions(product_id, &batches, 8);

    assert!(matches!(
        result,
        Err(StoreError::InsufficientInventory(id, 8, 7)) if id == product_id
    ));
}

#[test]
fn writeoff_on_never_stocked_product_reports_zero_available() {
    let product_id = Uuid::new_v4();

    let result = build_fifo_deductions(product_id, &[], 1);

    assert!(matches!(
        result,
        Err(StoreError::InsufficientInventory(id, 1, 0)) if id == product_id
    ));
}

#[test]
fn sold_out_oldest_batch_is_never_deducted() {
    let product_id = Uuid::new_v4();
    let sold_out = make_batch(product_id, 10, 0);
    let live = make_batch(product_id, 5, 5);
    let live_id = live.id;

    let deductions = build_fifo_deductions(product_id, &[sold_out, live], 5).unwrap();

    assert_eq!(deductions, vec![(live_id, 5)]);
}

#[test]
fn batch_picker_short_id_is_prefix_of_full_batch_id() {
    let row = make_batch(Uuid::new_v4(), 5, 5);
    let id_string = row.id.to_string();
    let received_at = row.received_at;

    let info = InventoryBatchInfo::from(row);

    assert_eq!(info.short_id.len(), 8);
    assert!(id_string.starts_with(&info.short_id));
    assert_eq!(info.quantity_received, 5);
    assert_eq!(info.quantity_remaining, 5);
    assert_eq!(info.received_at, received_at);
}
