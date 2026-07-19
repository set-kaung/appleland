# Appleland

Retail management backend API built with Rust/Axum.

## Tech Stack

- **Runtime**: Rust (edition 2024) + Tokio
- **HTTP**: Axum 0.8
- **Database**: PostgreSQL via sqlx (compile-time checked queries)
- **Auth**: Supabase JWT (ES256) for users, session-based (Argon2) for admins
- **Docs**: utoipa (auto-generated OpenAPI specs)
- **Infra**: GitLab CI, rate limiting (tower_governor)

## Project Structure

```
src/
├── main.rs              # Router setup, server bootstrap
├── config.rs            # Env config (envy) + CLI args (clap)
├── db.rs                # PgPool, DBQueryFilter trait
├── middleware.rs        # Auth, CORS, rate limiting, tracing
├── auth/                # SupabaseAuth (JWT/JWKS), admin password/session
├── product/             # CRUD, search (simple + advanced), pagination
├── category/            # Admin-only category management
├── inventory/           # Inventory records (WIP)
├── sales/               # Admin-only sales read endpoints
└── order/
```

## API Endpoints

For more specifics, refer to openapi.yaml files under docs folder.

**User** (`/api/`) — Supabase JWT required:

| Method | Path                   | Description                     |
| ------ | ---------------------- | ------------------------------- |
| GET    | `/api/products`        | Paginated active products       |
| GET    | `/api/products/{id}`   | Product by ID                   |
| GET    | `/api/products/search` | Search by name/barcode/category |

**Admin** (`/admin/`) — Session auth required:

| Method   | Path                       | Description                       |
| -------- | -------------------------- | --------------------------------- |
| POST     | `/admin/login`             | Login (creates session)           |
| POST     | `/admin/logout`            | Destroy session                   |
| GET      | `/admin/products`          | Paginated products (all statuses) |
| GET      | `/admin/products/all`      | All products (SaleAdmin+)         |
| POST     | `/admin/products`          | Create product                    |
| PUT      | `/admin/products/{id}`     | Update product                    |
| DELETE   | `/admin/products/{id}`     | Delete product                    |
| GET/POST | `/admin/products/search/*` | Simple + advanced search          |
| GET/POST | `/admin/categories`        | List / create categories          |
| GET      | `/admin/sales`             | Paginated sales                   |
| GET      | `/admin/sales/{id}`        | Sale detail with line items       |

**Health**: `GET /health`

## Database

PostgreSQL with 12 migrations in `db/migrations/`. Full schema at `db/schema.sql`.

**Tables**: `admins`, `categories`, `products`, `inventory`, `sales`, `sale_items`, `orders`, `shipping_addresses`

**Roles**: `sale_admin`, `playground_admin`, `super_admin`

## Configuration

Required env vars

| Variable                  | Description                    |
| ------------------------- | ------------------------------ |
| `DATABASE_URL`            | PostgreSQL connection string   |
| `SUPABASE_REF`            | Supabase project reference     |
| `PORT`                    | Server port                    |
| `ALLOWED_ORIGINS`         | CORS origins (comma-separated) |
| `PER_SEC`                 | Rate limit: req/sec            |
| `BURST_SIZE`              | Rate limit: burst size         |
| `SESSION_DELETE_INTERVAL` | Session cleanup interval       |
| `RUST_LOG`                | Log level                      |
| `SQLX_OFFLINE`            | sqlx offline mode              |
| `API_VERSION`             | API version identifier         |

## Run

```bash
./run.sh .local.env
```

Or manually:

```bash
export $(cat .local.env | xargs) && cargo run --release
```

## Build

```bash
cargo build --release
# Binary: target/release/appleland
# Install: ./install.sh (copies to ~/bin/)
```

## Docker

```bash
docker build -t appleland .
docker run -p 3010:3010 --env-file .env appleland
```

## API Docs

Generated at startup to `docs/openapi.yaml` and `docs/admin/openapi.yaml`.

```bash
./target/release/appleland --gen-docs-only  # generate without starting server
```
