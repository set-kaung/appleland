# Technical Stack Overview & Reasoning

### Language: Rust (Edition 2024)

Rust was chosen for several reasons:

- **Memory safety without garbage collection** — critical for a long-running server handling concurrent requests. No GC pauses means predictable latency under load.
- **Fearless concurrency** — the borrow checker enforces thread safety at compile time, eliminating data races in the async runtime.
- **Performance** — compiled to native code, performance comparable to C/C++. For a retail backend that may handle inventory queries, sales processing, and concurrent admin sessions, this ensures responsiveness.
- **Ecosystem maturity** — `sqlx`, `axum`, and `tokio` provide a production-grade async stack with compile-time query checking and ergonomic error handling.

### HTTP Framework: Axum 0.8.9

Axum is a modular, Tower-based web framework. Key reasons for selection:

- **Tower middleware compatibility** — Axum is built on Tower, giving access to a rich ecosystem of middleware (rate limiting, CORS, timeouts, tracing) without custom implementations.
- **Extractor pattern** — type-safe request extraction (path params, query params, JSON body, session, auth claims) via Rust's type system. Invalid requests are rejected at the framework level before reaching handler logic.
- **Compile-time route safety** — handler signatures enforce correct parameter types and state injection at compile time.
- **Lightweight** — minimal magic, explicit composition. Router nesting and layer application are straightforward.

### Runtime: Tokio

The async runtime powering all I/O operations. Tokio was chosen because:

- **Industry standard** — the most widely used Rust async runtime with extensive community support.
- **Full-featured** — provides TCP listeners, signal handling, task spawning with abort handles, timers, and synchronization primitives.
- **Graceful shutdown** — `tokio::select!` with `signal::ctrl_c()` and `SIGTERM` handling ensures clean shutdown of the session cleanup background task.

### Database: PostgreSQL via Supabase

**Current hosting**: Supabase (managed PostgreSQL, AWS ap-northeast-1 region).

**Driver**: `sqlx` 0.9.0 with compile-time checked queries.

- `sqlx::query_as!` and `sqlx::query_scalar!` macros validate SQL against the actual database schema at compile time (via `.sqlx/` offline metadata). This catches column name mismatches, type errors, and missing tables before deployment.
- **TLS**: `tls-rustls-ring-webpki` — Rust-native TLS stack, no OpenSSL system dependency required. Simplifies deployment across environments.
- **Connection pool**: `PgPoolOptions` with 10-second statement timeout, application-level query logging via `tracing`.

**Schema design** — 9 domain tables plus a session store:

| Table                           | Purpose                                                             |
| ------------------------------- | ------------------------------------------------------------------- |
| `admins`                        | Admin users with Argon2 password hashes and role enum               |
| `categories`                    | Self-referencing product categories (sub-categories via FK)         |
| `products`                      | Core product data: barcode (unique), name, selling_price, is_active |
| `inventory`                     | Per-product stock tracking with `CHECK (quantity_remaining >= 0)`   |
| `sales` / `sale_items`          | Sales records with line items linked to inventory                   |
| `orders` / `shipping_addresses` | Order fulfillment with address management                           |
| `tower_sessions.session`        | Server-side session storage (PostgreSQL-backed)                     |

**Monetary values**: `NUMERIC(19,2)` / `NUMERIC(20,2)` via `rust_decimal` crate — fixed-precision arithmetic avoids floating-point rounding errors in price calculations.

**Migrations**: 12 sequential SQL migrations in `db/migrations/`, applied at deployment time.

### Hosting Strategy

**Current**: Supabase provides managed PostgreSQL, eliminating database administration overhead (backups, patching, connection pooling).

**Planned migration**: AWS or DigitalOcean for the Rust backend deployment.

- **DigitalOcean**: Simpler operational model, suitable for smaller deployments. Droplets or App Platform provide straightforward Rust binary hosting.
- **AWS**: ECS/Fargate for containerized deployment, or EC2 for direct binary hosting. More granular scaling control if traffic grows.
- The backend is a single self-contained binary (`appleland`) with no system dependencies beyond libc and TLS root certificates, making deployment straightforward on either platform.

### Authentication: Dual System

The application serves two distinct user populations with different auth mechanisms:

#### User Auth — Supabase JWT (Bearer Token)

- **Flow**: Client obtains JWT from Supabase Auth (OAuth/email) → sends `Authorization: Bearer <token>` on API requests → middleware verifies against Supabase JWKS endpoint.
- **Algorithm**: ES256 (ECDSA with P-256 + SHA-256).
- **JWKS caching**: Fetched keys cached with 10-minute TTL to avoid per-request external calls.
- **Scope**: Applied to all `/api/*` routes (product browsing, search).

**Reasoning**: Offloading user authentication to Supabase eliminates the need to manage user registration, password reset, OAuth provider integration, and token refresh flows. The Rust backend only needs to verify tokens — a stateless operation.

#### Admin Auth — Password + Session

- **Flow**: Admin sends credentials → Argon2id password verification → server-side session created in PostgreSQL → session cookie sent to client.
- **Password hashing**: Argon2id — winner of the Password Hashing Competition, resistant to GPU-based attacks.
- **Session storage**: `tower-sessions` with PostgreSQL backend (`tower_sessions.session` table). Sessions expire after 24 hours of inactivity.
- **Role-based access**: `AdminRole` enum (`sale_admin`, `playground_admin`, `super_admin`) with a `RequiresRole<T>` extractor that enforces permissions per-handler. `SuperAdmin` bypasses all role checks.
- **Scope**: Applied to all `/admin/*` routes.

**Reasoning**: Admin sessions use server-side storage (unlike JWTs) because:

1. Sessions can be revoked instantly (unlike stateless JWTs which persist until expiry).
2. Admin actions are sensitive — server-side validation ensures the session hasn't been tampered with.
3. Role checks happen at the extractor level, keeping handler logic clean.

### Middleware Stack

Applied globally in this order (outermost first):

| Layer                  | Purpose                                                      |
| ---------------------- | ------------------------------------------------------------ |
| `TraceLayer`           | Request/response tracing via `tracing`                       |
| `debug_log_middleware` | Custom — logs `{status} {method} {path}` for every request   |
| `CorsLayer`            | Configurable origins, methods, headers; credentials allowed  |
| `TimeoutLayer`         | 10-second request timeout → 504 Gateway Timeout              |
| `GovernorLayer`        | Token-bucket rate limiting (configurable burst + per-second) |

**Rate limiting**: `tower_governor` with configurable `BURST_SIZE` (default: 15) and `PER_SEC` (default: 10). Rejects requests before they reach handlers, protecting backend resources under load.

### Configuration Management

Environment variables deserialized directly into a `Config` struct via `envy`:

| Variable                  | Purpose                                                |
| ------------------------- | ------------------------------------------------------ |
| `DATABASE_URL`            | PostgreSQL connection string                           |
| `SUPABASE_REF`            | Supabase project reference (for JWKS URL construction) |
| `ALLOWED_ORIGINS`         | Comma-separated CORS origins                           |
| `PORT`                    | Server listen port (default: 8080)                     |
| `BURST_SIZE` / `PER_SEC`  | Rate limit parameters                                  |
| `SESSION_DELETE_INTERVAL` | Expired session cleanup interval                       |

CLI flags via `clap`: `--gen-docs-only` (`-g`) generates OpenAPI docs and exits without starting the server or connecting to the database.

### API Documentation

Auto-generated from code annotations using `utoipa`:

- **User API**: `docs/openapi/user/openapi.yaml` — 3 endpoints (product browsing, search).
- **Admin API**: `docs/openapi/admin/openapi.yaml` — 15 endpoints (CRUD, inventory, sales, auth).
- Generated at startup, or standalone via `--gen-docs-only`.
- Types annotated with `#[derive(ToSchema)]`, endpoints with `#[utoipa::path(...)]` — documentation stays in sync with code changes.

### CI/CD (GitLab CI)

- **Build job**: `rust:1.97-slim` image, `cargo build` + `cargo test` with `SQLX_OFFLINE=true`.
- **Security scanning**: SAST and secret detection via GitLab templates.
- **Caching**: `target/` directory cached per branch to speed up incremental builds.
- **Policy**: `cargo deny` configured for license compliance (MIT, Apache-2.0, BSD variants allowed).

### Error Handling

Three-layer error taxonomy:

| Layer          | Type         | Purpose                                                                            |
| -------------- | ------------ | ---------------------------------------------------------------------------------- |
| Startup/Config | `AppError`   | Fatal errors during initialization                                                 |
| Database       | `StoreError` | Query failures with Postgres error code mapping (e.g., `23505` → unique violation) |
| HTTP           | `APIError`   | Client-facing errors with appropriate status codes                                 |

This separation ensures database-level errors are properly translated to HTTP responses without leaking internal details.
