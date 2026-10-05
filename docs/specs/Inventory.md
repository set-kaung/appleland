## Sale Tracking

Each entry in the inventory table/store is each instance of stocking by the bussiness.
Each entry has associated quantity stocked for that entry and the unit cost.
When selling, entires with non-zero `quantity_remaining` will be deducted
in FIFO (first in first out) order.

## Perishable

Some inventory are perishable. There is `is_perishable` column on `products`.

### Expiring-soon analytics

`GET /admin/analytics/ai/expiring-soon?days=30` requires a SuperAdmin session.
It returns an unpaginated JSON array of all batches with `quantity_remaining > 0`
and an expiry date from today through today + `days`, inclusive, in `Asia/Yangon`.
`days` defaults to 30, accepts integers from 0 to 65535, and `days=0` means today only.
Invalid values return HTTP 400. Batches without an expiry date or with an expiry
date before today are excluded. Products are included even if soft-deleted, so
remaining stock is still visible.

Each row contains `product_id`, `product_name`, `batch_id` (the inventory record ID),
`quantity_remaining`, and `expiry_date` (RFC 3339 timestamp). Results are ordered
by expiry timestamp ascending, then batch ID ascending. No matches returns `[]`.
