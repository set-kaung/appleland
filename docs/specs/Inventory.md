## Sale Tracking

Each entry in the inventory table/store is each instance of stocking by the bussiness.
Each entry has associated quantity stocked for that entry and the unit cost.
When selling, entires with non-zero `quantity_remaining` will be deducted
in FIFO (first in first out) order.

## Perishable

Some inventory are perishable. There is `is_perishable` column on `products`.
