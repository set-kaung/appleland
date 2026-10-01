# Playground Tickets

## Overview

The playground feature manages physical playground ticket sales through claim tokens.
Admins create claim tokens; users claim them via the frontend, optionally redeeming
playground coupons for free tickets.

## Core Tables

- `playground_purchase_tokens` — claim tokens created by admins.
  - Lifecycle: `pending` → `claimed` (by a user) or `expired` (sweeper).
  - Stores `total_quantity`, `unit_price`, `claimed_by`, `claimed_at`, and
    `free_quantity` (number of tickets redeemed via coupons).
- `playground_purchases` — canonical purchase rows produced when a token is claimed.
  - Each claim creates one paid row and/or one free-redemption row.
  - `token_id` references `playground_purchase_tokens(id)` with a strong foreign key.
- `playground_accounts` — per-user running totals of purchased and redeemed tickets.
- `playground_coupons` — free-ticket coupons earned through the loyalty program.

## Token Claim Flow

1. Admin creates a token (`POST /admin/playground/tokens`).
2. User polls or views checkout for the token.
3. User claims the token (`PATCH /api/playground/tokens/{token_id}`), optionally
   supplying coupon IDs.
4. The claim transaction:
   - Marks the token `claimed` and records `free_quantity`.
   - Updates the user's `playground_accounts` totals.
   - Marks redeemed coupons as used.
   - Inserts paid and/or free `playground_purchases` rows linked to the token.

## Pricing on the Admin Checkout Endpoint

`GET /admin/playground/checkout/{token_id}` is polled by the admin UI.

- Before claim: the endpoint returns `claim: null`.
- After claim: the endpoint returns `claim` containing the claiming user and the
  total price (computed from `total_quantity - free_quantity`).

This avoids the earlier bug where post-claim pricing was recomputed from the user's
*remaining* available coupons, which undercounted redeemed coupons and overstated
the charge.

## Data Retention and Cleanup

`playground_purchases.token_id` has a strong foreign key to
`playground_purchase_tokens(id)`. Because of this constraint, **a token cannot be
deleted while it still has purchase rows referencing it**.

Implications:

- Do not delete individual claimed tokens to free space; the FK will block it.
- The existing token sweeper only deletes `expired` tokens and marks pending tokens
  as expired. It does not touch claimed tokens.
- If the business later needs to trim or reset playground data, the supported
  approach is to archive/export both tables and then truncate or recreate them
  together:

  ```sql
  -- Example: archive both tables and start fresh
  TRUNCATE TABLE playground_purchases, playground_purchase_tokens RESTART IDENTITY CASCADE;
  ```

  (Also truncate/reset `playground_accounts` and `playground_coupons` if a full
  reset is desired.)

If partial retention is needed in the future, remove or relax the FK constraint
before implementing a token-only deletion job.
