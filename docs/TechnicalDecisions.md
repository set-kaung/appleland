# Appleland Backend — Technical Decisions & Architecture Reference

This document captures the deep technical decisions made during the design and implementation of the Appleland backend. It is intended as a reference for understanding *why* the system is built the way it is, not as a high-level overview.

---

## Table of Contents

1. [Language, Runtime & Build Configuration](#1-language-runtime--build-configuration)
2. [Layered Module Architecture](#2-layered-module-architecture)
3. [Dual Authentication System](#3-dual-authentication-system)
4. [Role-Based Access Control](#4-role-based-access-control)
5. [Data Flow & Transaction Orchestration](#5-data-flow--transaction-orchestration)
6. [Error Handling Taxonomy](#6-error-handling-taxonomy)
7. [Security Decisions](#7-security-decisions)
8. [Database Design Decisions](#8-database-design-decisions)
9. [FIFO Inventory Costing](#9-fifo-inventory-costing)
10. [API Design Patterns](#10-api-design-patterns)
11. [Payment Integration](#11-payment-integration)
12. [Background Tasks & Graceful Shutdown](#12-background-tasks--graceful-shutdown)
13. [AI Chat Relay](#13-ai-chat-relay)
14. [Image Storage](#14-image-storage)
15. [Configuration Management](#15-configuration-management)
16. [Logging & Observability](#16-logging--observability)
17. [Testing Strategy](#17-testing-strategy)
18. [CI/CD & sqlx Offline Cache](#18-cicd--sqlx-offline-cache)
19. [Trade-offs Summary](#19-trade-offs-summary)

---

## 1. Language, Runtime & Build Configuration

### 1.1 Rust (Edition 2024)

**Decision**: Rust with edition 2024 was chosen as the implementation language.

**Rationale**:
- Memory safety without garbage collection — critical for a long-running server that must not experience GC pauses.
- The borrow checker enforces fearless concurrency at compile time, preventing data races in async code.
- Native performance comparable to C++ for I/O-bound workloads.
- Ecosystem maturity: `axum` for HTTP, `sqlx` for database access, `tokio` for async runtime, `utoipa` for OpenAPI generation.

**Trade-off**: Steeper learning curve and longer compile times compared to GC'd languages (Go, Java, Node.js). The compile-time guarantee of memory safety is worth the productivity cost for a system that must run continuously without degradation.

### 1.2 Crate-Level Lint Gates

**File**: `src/lib.rs:1-18`, `src/main.rs:1-7`

**Decision**: The following lints are set to `deny` at the crate root:

```rust
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
```

Additional warnings enabled:
- `clippy::dbg_macro` — no debug prints in production code
- `clippy::todo` / `clippy::unimplemented` — no unfinished code paths
- `clippy::redundant_clone` — catch unnecessary allocations
- `clippy::needless_collect` — catch unnecessary intermediate collections
- `clippy::inefficient_to_string` — catch suboptimal string conversions
- `clippy::large_futures` — catch futures that are too large (stack overflow risk)
- `clippy::await_holding_lock` — catch holding a lock across an await point (deadlock risk)

**Rationale**: These lints enforce explicit error handling. No `.unwrap()`, no `.expect()`, no `panic!`, no direct indexing/slicing in production code. All error paths must be handled explicitly. This is non-negotiable for a production system.

**Note**: `clippy::pedantic` is commented out (`src/lib.rs:1`) to avoid overly noisy lints that would slow development without catching real bugs.

**Test files** use `#![allow(clippy::unwrap_used, clippy::indexing_slicing)]` (e.g., `src/sales/tests.rs:1`, `src/inventory/tests.rs:1`) since test code is allowed to panic on failure.

### 1.3 Release Profile Optimization

**File**: `Cargo.toml:7-11`

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

**Rationale**: Maximum optimization (`opt-level = 3`), link-time optimization across all crates (`lto = "fat"`), single codegen unit for maximum inlining (`codegen-units = 1`), and stripped binary (`strip = true`). The result is a single self-contained binary with no system dependencies beyond libc and TLS root certificates. Compile time is sacrificed for runtime performance.

### 1.4 Key Dependencies

**File**: `Cargo.toml:13-61`

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `axum` | 0.8.9 | HTTP framework with `macros` and `multipart` features |
| `sqlx` | 0.9.0 | Compile-time checked SQL with `runtime-tokio`, `tls-rustls-ring-webpki`, `postgres`, `uuid`, `rust_decimal`, `time`, `macros` |
| `tokio` | 1.52.3 | Async runtime with `full` features |
| `tower-sessions` | 0.15.0 | Session management |
| `tower-sessions-sqlx-store` | git | PostgreSQL-backed session store |
| `argon2` | 0.5.3 | Password hashing (Argon2id) |
| `utoipa` | 5.5 | OpenAPI documentation generation |
| `clap` | 4.6.1 | CLI argument parsing |
| `rust_decimal` | — | Fixed-precision decimal arithmetic for monetary values |
| `hmac`, `sha2`, `hex` | — | Payment webhook signature verification |
| `jsonwebtoken` | 10.4.0 | JWT verification with `rust_crypto` |
| `reqwest` | 0.13.3 | HTTP client with `rustls` TLS |

**Notable**: `tls-rustls-ring-webpki` is used instead of OpenSSL to avoid the OpenSSL system dependency, making the binary truly self-contained.

---

## 2. Layered Module Architecture

### 2.1 Store / Handler / Service Separation

**Files**: `docs/codebaseguide/service-layer.md`, `AGENTS.md`

**Decision**: Each feature module (`product`, `category`, `inventory`, `sales`, `orders`, `playground`, `users`, `analytics`, `ai`, `staff`, `admin_settings`) splits into four files:

| File | Responsibility |
|------|---------------|
| `mod.rs` | Route definitions only — axum `Router<AppState>` with path/ method bindings |
| `handler.rs` | Thin layer: input validation, auth/actor handling, orchestrating service calls, mapping results to HTTP responses |
| `model.rs` | DTOs (request/response structs), filter structs, database row structs |
| `store.rs` | Data access ONLY — logic-agnostic, one SQL query per function, no transactions, no business logic |
| `service.rs` | All real business logic AND orchestration, including transactions: open/commit/rollback and sequencing across multiple store calls |

**Rationale**: This strict separation ensures:
- Store functions are composable and testable in isolation.
- Handlers stay thin — they don't contain business logic.
- Business logic lives in one place (`service.rs`), making it easy to find and modify.
- Transactions are opened/committed/rolled back in the service layer, never in store functions.

**Consequence**: More files and boilerplate compared to a monolithic handler approach, but the separation pays off in maintainability and testability.

### 2.2 Feature Module Organization

**File**: `src/lib.rs:23-45`

13 feature modules are declared as top-level modules. Each owns its domain completely. Cross-module calls go through the service layer (e.g., `sales::service` calls `inventory::store`, `orders::service`, `users::store`).

### 2.3 AppState Dependency Injection

**File**: `src/app_state.rs:21-49`

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub storage: StorageConfig,
    pub payment: Arc<dyn PaymentProvider>,
    pub demo_mode: bool,
    pub ai: AiConfig,
}
```

**Decision**: `AppState` is a `Clone` struct holding all shared dependencies. All handlers extract `State<AppState>`. Trait objects (`dyn AuthProvider`, `dyn PaymentProvider`) enable swapping implementations (e.g., Supabase vs. other auth, MMPay vs. other payment) without changing handler code.

### 2.4 Route Assembly

**File**: `src/routes.rs:53-133`

Routes are assembled in `create_app()` with clear separation:

```
/health           — unauthenticated health check
/api/**           — user routes behind user_auth_middleware (JWT) + user_row_middleware
/admin/**         — admin routes behind admin_auth_middleware (session cookie)
/admin/ai/chat    — AI route placed OUTSIDE global middleware (no rate limit, no 10s timeout)
/payment          — webhook route from payment provider (outside auth middleware)
```

**Rationale**: The AI chat endpoint is placed outside the global middleware stack because Gemini responses can take longer than the 10-second timeout and should not be rate-limited.

---

## 3. Dual Authentication System

### 3.1 User Authentication (Supabase JWT)

**Files**: `src/middleware.rs`, `src/auth/supabase.rs`

**Decision**: User routes use Supabase JWT (ES256) verified against the JWKS endpoint.

**Implementation details**:
- JWKS is cached with a 10-minute TTL.
- **Inflight deduplication**: When multiple requests arrive simultaneously and the JWKS cache is stale, only one HTTP request is made to Supabase. Others wait on a `tokio::sync::broadcast::Receiver`. The inflight slot is cleared before broadcasting to prevent late subscribers from missing the value.
- Claims are inserted into request extensions for downstream handlers.
- The `user_row_middleware` resolves the Supabase UUID to an internal user ID on every request. If no user record exists, returns `403 AccountSetupRequired`.

**Rationale**: User auth is stateless — Supabase manages registration, OAuth, and token refresh. The backend only verifies the JWT and resolves the user ID.

### 3.2 Admin Authentication (Session Cookie)

**Files**: `src/auth/admin/handler.rs`, `src/middleware.rs:131-157`

**Decision**: Admin routes use Argon2id password verification with server-side sessions in PostgreSQL via `tower-sessions`.

**Implementation details**:
- Session cookie named `id`, `SameSite=None`, `Secure`, 24-hour inactivity expiry.
- `partitioned_cookie_middleware` appends `; Partitioned` to the session cookie for third-party iframe/POS embedding.
- **Session ID rotation**: After login (privilege escalation), `cycle_id()` is called to prevent session fixation attacks.
- Background task continuously deletes expired sessions.
- **Timing side-channel prevention**: When admin username is not found, the system performs a dummy Argon2 hash with a fixed zero salt to burn time, making login attempts take roughly the same duration regardless of whether the user exists.

**Rationale**: Admin auth is stateful for instant revocation and role-based access control. Sessions can be invalidated immediately by deleting the session record.

### 3.3 Why Two Systems?

| Aspect | User Auth (JWT) | Admin Auth (Session) |
|--------|-----------------|---------------------|
| State | Stateless | Stateful |
| Revocation | Requires token expiry or Supabase-side invalidation | Immediate (delete session record) |
| Registration | Managed by Supabase (OAuth, email, etc.) | Manual (admin creates accounts) |
| Token refresh | Handled by Supabase | N/A (session cookie) |
| Use case | End users on mobile/web | Admin staff on web dashboard |

---

## 4. Role-Based Access Control

### 4.1 AdminRole Enum

**Files**: `src/app_extractor.rs:36-94`, `src/auth/admin/model.rs:9-27`

```rust
pub enum AdminRole {
    SaleAdmin,
    PlaygroundAdmin,
    SuperAdmin,
}
```

Stored as a PostgreSQL enum. Three marker types enforce permissions at the type level:

```rust
pub struct SaleAdminRole;
pub struct PlaygroundAdminRole;
pub struct SuperAdminRole;
```

### 4.2 RequiresRole Extractor

**Decision**: `RequiresRole<T>` extractor enforces permissions per-handler via `FromRequestParts`. `SuperAdmin` bypasses all role checks.

**Rationale**: Role enforcement is declarative — the handler signature itself documents what role is required. The extractor runs before the handler body, so unauthorized requests are rejected before any business logic executes.

---

## 5. Data Flow & Transaction Orchestration

### 5.1 Request Lifecycle

```
1. Global middleware stack (rate limit -> timeout -> CORS -> request ID -> tracing)
2. Auth middleware (JWT verification or session validation)
3. Role extraction (RequiresRole<T>)
4. Handler (validation, orchestration)
5. Service layer (business logic, transactions)
6. Store layer (single SQL queries)
7. Response mapping (From<Row> -> DTO)
```

### 5.2 Transaction Orchestration

**Files**: `src/sales/service.rs:73-131`, `src/inventory/service.rs:23-54`, `src/playground/service.rs:18-147`, `src/orders/service.rs:47-114`

**Decision**: Transactions are always opened in the service layer. Store functions accept `&mut Transaction<'_, Postgres>` or a generic `Executor` trait bound. The service sequences multiple store calls and commits/rolls back atomically.

**Pattern**:
```rust
let mut tx = pool.begin().await?;
// ... multiple store:: calls on &mut tx ...
tx.commit().await?;
// On error, the transaction is dropped (implicit rollback)
```

**Rationale**: Store functions are logic-agnostic and composable. The service layer owns the transaction boundary, ensuring atomicity across multiple store calls.

### 5.3 Generic Executor Pattern

**Files**: `src/sales/store.rs:180-220`, `src/product/store.rs:141-179`, `src/inventory/store.rs:9-35`, `src/users/store.rs:53-74`

**Decision**: Store functions that only read use `E: Executor<'e, Database = Postgres>` generic bound, allowing them to be called with either `&PgPool` or `&mut Transaction`. Write functions that are called within transactions accept `&mut Transaction<'_, Postgres>` directly.

**Rationale**: Read functions can be used both inside and outside transactions without duplication. Write functions are explicitly transaction-bound, preventing accidental use outside a transaction.

### 5.4 Parallel Query Execution

**Files**: `src/sales/service.rs:289-306`, `src/orders/service.rs:146-154,164-180`, `src/playground/service.rs:185-200`

**Decision**: `tokio::try_join!` is used to run independent queries concurrently (e.g., fetching sale row + sale items + sale admin in parallel). This reduces latency for read-heavy detail endpoints.

### 5.5 DTO Mapping via From Trait

**Decision**: Database row structs (`ProductRow`, `SaleRow`, `AdminSaleListRow`, etc.) are separate from API response structs (`UserProduct`, `AdminProduct`, `AdminSale`, etc.). `From<Row>` impls convert between them.

**Rationale**: Decouples the database schema from the API contract. Changes to the database schema don't automatically leak into the API response.

---

## 6. Error Handling Taxonomy

### 6.1 Three-Layer Error Types

**File**: `src/app_error.rs:1-204`

```
AppError    — startup/config errors (fatal)
StoreError  — database errors with Postgres error code mapping
APIError    — client-facing HTTP errors
```

**Conversion chain**: `sqlx::Error` -> `StoreError` (via `From` impl, lines 138-164) -> `APIError` (via `From` impl, lines 166-189).

**Rationale**: Database errors are translated to appropriate HTTP responses without leaking internal details. The conversion chain ensures that every error path results in a meaningful HTTP response.

### 6.2 Postgres Error Code Mapping

**File**: `src/app_error.rs:138-164`

| Postgres Code | StoreError Variant |
|---------------|-------------------|
| `23505` (unique violation) | `StoreError::UniqueKeyViolated` |
| `23502` (not null violation) | `StoreError::NullField` |
| `23503` (foreign key violation) | `StoreError::RecordNotFound` |
| `RowNotFound` | `StoreError::RecordNotFound` |

### 6.3 APIError HTTP Status Mapping

**File**: `src/app_error.rs:103-136`

| APIError Variant | HTTP Status |
|-----------------|-------------|
| `InternalErr` | 500 |
| `AdminInvalidCreds` | 401 |
| `Forbidden` | 403 |
| `AccountSetupRequired` | 403 |
| `BadRequest` | 400 |
| `Unauthorized` | 401 |
| `NotFound` | 404 |
| `TokenExpired` | 410 (Gone) |
| `InvalidCoupons` | 400 |
| `PaymentNotConfirmed` | 402 (Payment Required) |
| `TooManyRequests` | 429 |
| `GatewayTimeout` | 504 |
| `Conflict` | 409 |

### 6.4 ValidatedJson Extractor

**File**: `src/app_extractor.rs:13-34`

**Decision**: Custom `ValidatedJson<T>` extractor wraps `Json<T>` and converts parse errors to `422 Unprocessable Entity` with a generic "Invalid request body" message, preventing serde error details from leaking to clients.

### 6.5 Timing Side-Channel Prevention

**File**: `src/auth/admin/handler.rs:15-17,71-87`

**Decision**: When admin username is not found, the system performs a dummy Argon2 hash with a fixed zero salt to burn time, making login attempts take roughly the same duration regardless of whether the user exists. This prevents timing-based username enumeration.

---

## 7. Security Decisions

### 7.1 SQL Injection Prevention

**Files**: All `store.rs` files, `src/db.rs:35-44`

**Decision**: All queries use sqlx's compile-time checked macros (`query!`, `query_as!`, `query_scalar!`) with bound parameters. Dynamic filters use `QueryBuilder` with `push_bind()`. The `escape_like()` function escapes `%`, `_`, and `\` in LIKE patterns to prevent wildcard injection.

### 7.2 Password Hashing

**File**: `src/auth/admin/handler.rs:5-8,45-47`

**Decision**: Argon2id (default parameters) for admin password hashing. Passwords are never stored in plain text.

### 7.3 Session Security

**Files**: `src/main.rs:123-126`, `src/middleware.rs:131-157`, `src/auth/admin/handler.rs:63`

- Session cookies are `Secure` and `SameSite=None` (for iframe/POS embedding).
- `partitioned_cookie_middleware` appends `; Partitioned` to the session cookie for third-party iframe contexts.
- Session ID is rotated (`cycle_id()`) after privilege escalation (login) to prevent session fixation.
- Sessions expire after 24 hours of inactivity.
- Background task continuously deletes expired sessions.

### 7.4 Payment Webhook Security

**Files**: `src/payment/mmp.rs:278-290,299-372`, `src/payment/store.rs`

- HMAC-SHA256 signature verification using webhook secret + nonce.
- Nonce replay protection via `payment_webhook_nonces` table with `ON CONFLICT DO NOTHING`.
- Duplicate nonces are acknowledged (200 OK) to stop provider retries.
- Expired nonces cleaned up by background task (24h retention).

### 7.5 Rate Limiting

**File**: `src/middleware.rs:159-259`

**Decision**: `tower_governor` with configurable `BURST_SIZE` (default 15) and `PER_SEC` (default 10). Applied globally to all routes except the AI chat endpoint.

### 7.6 Request Timeout

**File**: `src/middleware.rs:178-189`

**Decision**: 10-second `TimeoutLayer` on all routes except AI chat. Returns 504 Gateway Timeout on expiry.

### 7.7 CORS

**File**: `src/middleware.rs:190-202`, `src/routes.rs:110-118`

**Decision**: Configurable allowed origins via `ALLOWED_ORIGINS` env var. Credentials allowed. Specific methods (GET, POST, PUT, PATCH, DELETE) and headers (CONTENT_TYPE, AUTHORIZATION, ACCEPT) allowed. Max age 3600s.

### 7.8 Request ID Propagation

**File**: `src/middleware.rs:111-129`

**Decision**: `x-request-id` header is extracted or generated (UUID v4), inserted into request extensions, and added to response headers. Used in tracing spans for request correlation.

---

## 8. Database Design Decisions

### 8.1 Monetary Values

**Files**: `db/migrations/0002_create_products.sql:10`, `db/migrations/0007_create_sales.sql:4`, `db/migrations/0008_create_saleitems.sql:8-9`

**Decision**: `NUMERIC(19,2)` for selling_price, cost_price, unit_cost; `NUMERIC(20,2)` for total_amount. `rust_decimal::Decimal` in Rust.

**Rationale**: Fixed-precision arithmetic avoids floating-point rounding errors. A `f64` cannot represent 0.1 exactly, which causes rounding errors in financial calculations. `NUMERIC(19,2)` provides 17 digits of integer precision and 2 decimal places, sufficient for any monetary value.

### 8.2 Soft Delete Pattern

**Files**: `db/migrations/0019_soft_delete_proudcts.sql`, `db/migrations/0027_category_soft_delete.sql`, `src/product/store.rs:241-261`, `src/category/store.rs:69-91`

**Decision**: Products and categories use soft delete (`is_deleted`, `deleted_at`, `deleted_by` columns). All queries filter `WHERE is_deleted IS FALSE`. Delete operations are actually UPDATEs.

**Rationale**: Preserves referential integrity and audit history. A hard delete would break foreign key references from sales, orders, and other tables. Soft delete allows recovery of accidentally deleted data.

**Cost**: Every query must include `WHERE is_deleted IS FALSE`. This is a deliberate trade-off for data safety.

### 8.3 Pricing Model

**Files**: `docs/specs/Products.md`, `db/migrations/0052_products_original_price.sql`

**Decision**: `selling_price` is the single authoritative price. `original_price` is a nullable compare-at price. `CHECK (original_price IS NULL OR original_price > selling_price)` constraint. Discount percentage is computed at render time, never stored. API validates `original_price > selling_price` with a 400 error.

**Rationale**: Storing a computed discount percentage would create a denormalized field that could become stale. Computing it at render time ensures consistency.

### 8.4 Full-Replace PUT Semantics

**Files**: `docs/specs/Products.md:22-38`, `src/product/store.rs:209-239`

**Decision**: `PUT /admin/products/{id}` is a full replace. The store `UPDATE` sets every column unconditionally. No `COALESCE` or partial-update logic. A missing `Option` field deserializes to `None`, silently clearing the stored value.

**Rationale**: Full replace is simpler and more predictable than partial update. The caller is expected to round-trip fields they are not changing. This is intentional and documented in the OpenAPI description.

**Trade-off**: Clients must send the complete resource on every PUT. A partial update would require `PATCH` semantics, which adds complexity.

### 8.5 Order Status State Machine

**Files**: `src/orders/model.rs:37-88`, `src/orders/service.rs:47-84`

**Decision**: `DeliveryStatus` enum with a linear state machine: `Pending -> OnDelivery -> Received`. The `next()` method returns `Err(AppError::InvalidOperation)` when trying to advance past `Received`. Status changes are recorded in `order_status_history` (append-only log with `changed_by`).

**Rationale**: A linear state machine prevents invalid transitions (e.g., `Received -> Pending`). The append-only history log provides an audit trail of who changed the status and when.

### 8.6 User-Supabase Identity Linkage

**Files**: `src/middleware.rs:75-109`, `src/users/service.rs`, `src/users/store.rs`

**Decision**: The `users` table has a `supabase_id` column linking to Supabase Auth. The `user_row_middleware` resolves the Supabase UUID to an internal user ID on every request. If no user record exists, returns `403 AccountSetupRequired`. The `/api/users/setup` endpoint creates the user record with contact info and initializes a playground account.

**Rationale**: Decouples the internal user ID from the Supabase UUID. The internal ID is stable even if the Supabase account is recreated.

### 8.7 Playground Token Lifecycle

**Files**: `docs/Playground.md`, `src/playground/store.rs:203-222`, `src/playground/mod.rs:23-40`

**Decision**: Claim tokens have a 1-hour expiry (`expires_at = NOW() + INTERVAL '1 hour'`). A background sweeper runs every 30 minutes: marks pending tokens as expired after a 30-minute grace period, then deletes expired tokens. Claimed tokens are never deleted (FK constraint from `playground_purchases.token_id`).

**Rationale**: The grace period allows for in-flight transactions that might still claim the token. Claimed tokens are preserved for audit purposes.

### 8.8 Database Connection Pool

**File**: `src/db.rs:8-26`

**Decision**: `PgPoolOptions` with 10-second statement timeout, `application_name = "appleland_api"`, and statement logging at Trace level. TLS via `tls-rustls-ring-webpki` (no OpenSSL dependency).

---

## 9. FIFO Inventory Costing

### 9.1 Overview

**Files**: `docs/specs/Inventory.md`, `src/inventory/store.rs:37-58`, `src/sales/service.rs:192-258`, `src/sales/tests.rs`

**Decision**: Inventory entries are individual stocking events with `quantity_remaining` and `unit_cost`. When selling, entries with non-zero `quantity_remaining` are deducted in FIFO order (ordered by `received_at ASC, id ASC`). The `build_sale_items` function splits deductions across multiple inventory batches when needed.

### 9.2 Why FIFO?

**Rationale**: FIFO (First In, First Out) is the most common inventory costing method. It assumes that the oldest inventory is sold first, which matches physical reality for most businesses (especially perishable goods). FIFO provides accurate cost tracking per batch and proper COGS (Cost of Goods Sold) calculation.

**Trade-off**: More complex than a single quantity column, but enables accurate cost tracking per batch. A single `quantity` column would lose the per-batch cost information.

### 9.3 Inventory Deduction Safety

**File**: `src/inventory/store.rs:60-88`

**Decision**: `deduct_inventory_batch` uses a single `UPDATE ... FROM (VALUES ...)` query with a `WHERE i.quantity_remaining >= d.deduct` guard. If `rows_affected != deductions.len()`, it returns `UnexpectedRowsAffected`. The sales service catches this and re-fetches fresh inventory to provide an accurate shortage error.

**Rationale**: The `WHERE` guard ensures that inventory cannot go negative. The `rows_affected` check catches race conditions where another transaction has already deducted the inventory.

### 9.4 Inventory Adjustment with Row Locking

**File**: `src/inventory/service.rs:56-115`, `src/inventory/store.rs:158-184`

**Decision**: Inventory adjustments use `SELECT ... FOR UPDATE` to lock rows before modifying. Additions require `inventory_id` (specific batch). Deductions can be from a specific batch or FIFO across all available batches. The `lock_fifo_inventory_batches` function locks all available batches in FIFO order.

**Rationale**: Row locking prevents race conditions when multiple transactions try to modify the same inventory batch simultaneously.

---

## 10. API Design Patterns

### 10.1 Generic Response Types

**File**: `src/lib.rs:47-50`, `docs/codebaseguide/response.md`

**Decision**: `ApiResult<T>`, `ApiResultWithStatusCode<T>`, and `PaginatedApiResult<T>` type aliases standardize responses. `PaginatedResponse<T>` includes `data`, `total_items`, `total_pages`, `current_page`.

### 10.2 Pagination

**File**: `src/pagination.rs:1-26`

**Decision**: `PaginationQuery` with optional `page` and `page_size`. Default page_size 10, clamped 1-100. Offset calculated as `(page - 1) * page_size` using `saturating_mul` to prevent overflow.

### 10.3 Dynamic Query Filtering

**Files**: `src/db.rs:28-33`, `docs/codebaseguide/filterparams.md`

**Decision**: `DBQueryFilter<Postgres>` trait with `apply_filter()` and `apply_sort()` methods. Filter structs implement this trait. `QueryBuilder` dynamically constructs WHERE clauses. Both data and count queries use the same filter to ensure consistency.

### 10.4 OpenAPI Documentation

**Files**: `src/api_doc/user.rs`, `src/api_doc/admin.rs`, `src/api_doc/mod.rs`

**Decision**: `utoipa` with `#[utoipa::path]` annotations on every handler. Docs generated at startup (or via `--gen-docs-only` CLI flag). Separate user and admin API docs. Security schemes: `bearer` for user API, `session_cookie` for admin API.

### 10.5 Idempotent Sale Creation

**File**: `src/sales/handler.rs:188-209`

**Decision**: User sale creation catches `StoreError::UniqueKeyViolated` on `payment_order_id` and returns the existing sale ID instead of failing. This makes the endpoint idempotent — retrying a sale with the same `payment_order_id` returns the original sale.

**Rationale**: Network failures can cause clients to retry. Idempotency ensures that a retry doesn't create a duplicate sale.

---

## 11. Payment Integration

### 11.1 MMPay Integration

**File**: `src/payment/mmp.rs:1-372`

**Decision**: Full MMPay payment provider implementation:
- Handshake -> Get payment info -> Check if paid (status == "SUCCESS")
- Sandbox and production base URLs
- HMAC-SHA256 request signing with nonce
- Webhook handler with signature verification and nonce replay protection
- Webhook route at `/payment` (outside auth middleware)

### 11.2 Payment Provider Trait

**File**: `src/payment/mod.rs:40-53`

```rust
pub trait PaymentProvider: Send + Sync {
    async fn handshake(&self) -> Result<String, AppError>;
    async fn get_payment_info(&self, order_id: &str) -> Result<PaymentInfo, AppError>;
    async fn is_paid(&self, order_id: &str) -> Result<bool, AppError>;
    fn provider_name(&self) -> &str;
    fn generate_signature(&self, payload: &str, nonce: &str) -> String;
    fn verify_signature(&self, payload: &str, nonce: &str, signature: &str) -> bool;
    fn webhook_route(&self) -> &'static str;
}
```

**Rationale**: Allows swapping payment providers without changing business logic. The sales service depends on the trait, not the concrete MMPay implementation.

### 11.3 Demo Mode

**File**: `src/sales/handler.rs:165-166`, `src/config.rs:25-26`, `AGENTS.md`

**Decision**: `DEMO_MODE=true` env var skips MMPay `is_paid`/amount checks, treats any `payment_order_id` as paid, stores `payment_method = "Demo"`. Added for project showcase; marked for removal after showcase day.

---

## 12. Background Tasks & Graceful Shutdown

### 12.1 Session Cleanup

**File**: `src/main.rs:108-116`

**Decision**: `PostgresStore::continuously_delete_expired()` spawned as a background task. Aborted on shutdown via `AbortHandle`.

### 12.2 Playground Token Sweeper

**File**: `src/playground/mod.rs:23-40`

**Decision**: Runs every 30 minutes (configurable). Marks pending tokens as expired after 30-minute grace period, then deletes expired tokens. Returns counts of deleted and marked tokens.

### 12.3 Payment Nonce Cleanup

**File**: `src/payment/mod.rs:55-72`

**Decision**: Runs every 3 hours (configurable). Deletes nonces older than 24 hours.

### 12.4 Graceful Shutdown

**File**: `src/main.rs:164-201`

**Decision**: `tokio::select!` on `ctrl_c()` and `SIGTERM`. All three background task abort handles are aborted on shutdown. The session deletion task is awaited to confirm cancellation.

---

## 13. AI Chat Relay

### 13.1 Gemini Proxy

**Files**: `src/ai/handler.rs`, `src/ai/service.rs`

**Decision**: The AI chat endpoint (`/admin/ai/chat`) relays the request body unchanged to Gemini's `generateContent` API and returns the response (status code, content-type, body) unchanged. This is a pure proxy — no prompt processing on the backend.

**Rationale**: The AI chat is a simple relay because the prompt engineering happens on the frontend. The backend only enforces authentication, rate limiting, and usage tracking.

### 13.2 Rate Limiting

**File**: `src/ai/service.rs:27-32`

**Decision**: Per-admin daily message limit (default 20, configurable via `ai_chat_daily_limit`). Usage tracked in `admin_ai_chat_usage` table with `ON CONFLICT DO UPDATE` atomic increment. Returns 429 when limit exceeded.

### 13.3 Timeout

**File**: `src/ai/service.rs:14`, `src/routes.rs:129-132`

**Decision**: 60-second upstream timeout for Gemini requests. The AI route is placed OUTSIDE the global middleware stack, so it is not subject to the 10-second `TimeoutLayer` or rate limiting.

---

## 14. Image Storage

### 14.1 Supabase Storage

**File**: `src/image_storage.rs:1-123`

**Decision**: Images uploaded to Supabase Storage via REST API. Filename is sanitized by appending a UUID to the original filename. `x-upsert: true` header allows overwriting. Returns a public URL. Uses the storage secret key for both `apikey` and `Authorization` headers.

**Rationale**: Supabase Storage provides a simple, scalable object storage solution that integrates with the existing Supabase auth infrastructure.

---

## 15. Configuration Management

### 15.1 Environment Variables

**File**: `src/config.rs:1-171`

**Decision**: `envy` crate deserializes env vars directly into `Config` struct.

**Required vars**: `DATABASE_URL`, `SUPABASE_REF`, `ALLOWED_ORIGINS`, `SUPABASE_STORAGE_REF`, `SUPABASE_STORAGE_URL`, `SUPABASE_STORAGE_SECRET_KEY`, `GEMINI_API_KEY`, `MMPAY_WEBHOOK_SECRET`, `MMPAY_PUBLISHABLE_KEY`.

**Optional vars with defaults**: `PORT=8080`, `BURST_SIZE=15`, `PER_SEC=10`, `SESSION_DELETE_INTERVAL=1m`, `PLAYGROUND_TOKEN_CLEANUP_INTERVAL=30m`, `WEBHOOK_NONCE_CLEANUP_INTERVAL=3h`, `MMPAY_SANDBOX=true`, `AI_CHAT_DAILY_LIMIT=20`, `GEMINI_MODEL=gemini-2.5-flash`.

### 15.2 Config Display

**File**: `src/config.rs:54-102`

**Decision**: Custom `Display` impl that logs config at startup but redacts sensitive values (shows `database_host` instead of full `database_url`, doesn't show secrets).

### 15.3 CLI

**File**: `src/config.rs:166-171`, `src/main.rs:59-65`

**Decision**: `clap` with `--gen-docs-only` / `-g` flag. Generates OpenAPI docs and exits without starting the server or connecting to the database.

---

## 16. Logging & Observability

### 16.1 Structured Logging

**File**: `src/main.rs:33-57`

**Decision**: `tracing` with `tracing-subscriber`. JSON format when `LOG_JSON=true`. Pretty JSON in dev mode. Env filter defaults to `appleland=info,tower_http=warn,axum::serve=warn`.

### 16.2 Request Tracing

**File**: `src/middleware.rs:204-257`

**Decision**: `TraceLayer` with custom span creation including `request_id`, `method`, `path`, `remote_addr`, `user_agent`, `x_forwarded_for`, `host`. Response logging includes status and latency. Failure logging includes error and latency.

### 16.3 Health Check

**File**: `src/routes.rs:26-51`

**Decision**: `/health` endpoint returns `{"server": "ok", "database": "ok|error"}` by pinging the database pool.

---

## 17. Testing Strategy

### 17.1 Unit Tests Only

**Files**: `src/sales/tests.rs`, `src/inventory/tests.rs`, `AGENTS.md`

**Decision**: Pure unit tests of FIFO inventory-costing logic (`build_sale_items` and `build_fifo_deductions`). No integration tests, no DB required. `cargo test` needs no database.

**Rationale**: The most complex and bug-prone logic is the FIFO inventory costing. Unit tests cover this logic thoroughly without requiring a database. Integration tests would require a running Postgres instance, which complicates CI.

### 17.2 Test Coverage

- `src/sales/tests.rs`: 5 tests covering single entry, FIFO split across entries, insufficient inventory, zero inventory, multiple products.
- `src/inventory/tests.rs`: 7 tests covering oldest-first deduction, spanning batches, full stock writeoff, over-request, never-stocked product, sold-out batch skipping, short ID prefix.

---

## 18. CI/CD & sqlx Offline Cache

### 18.1 GitLab CI

**File**: `.gitlab-ci.yml`, `docs/specs/TechnicalStack.md:130-135`

**Decision**: `rust:1.97-slim` image, `cargo build` + `cargo test` with `SQLX_OFFLINE=true`. SAST and secret detection. `target/` cached per branch. `cargo deny` for license compliance (MIT, Apache-2.0, BSD variants allowed).

### 18.2 sqlx Offline Cache

**File**: `AGENTS.md`, `.sqlx/` directory

**Decision**: Every `query!` macro is checked at compile time. CI builds with `SQLX_OFFLINE=true` using committed `.sqlx/query-*.json` cache. After editing SQL, must run `cargo sqlx prepare --workspace` to regenerate the cache.

**Rationale**: Compile-time SQL checking catches errors before deployment. The offline cache allows CI to build without a database connection.

---

## 19. Trade-offs Summary

| Decision | Benefit | Cost |
|----------|---------|------|
| Rust + Axum | Memory safety, performance, type safety | Steeper learning curve, longer compile times |
| sqlx compile-time checks | Catches SQL errors before deployment | Requires DB connection or offline cache for builds |
| Layered architecture (store/handler/service) | Separation of testability, maintainability | More files and boilerplate |
| Dual auth (JWT + session) | Stateless user auth, revocable admin sessions | Two auth systems to maintain |
| Soft delete | Data preservation, audit trail | All queries must filter `is_deleted` |
| FIFO inventory | Accurate COGS tracking | Complex deduction logic |
| Full-replace PUT | Simple, predictable semantics | Clients must round-trip all fields |
| Fixed-precision decimal | No floating-point errors | Slightly more complex than f64 |
| Background tasks | Automatic cleanup | Additional async tasks to manage |
| Demo mode | Easy showcase testing | Must be removed before production |
| Unit tests only | Fast, no DB required | No integration test coverage |
| AI chat as pure proxy | Simple backend, no prompt engineering | Limited backend control over AI behavior |
