# Authentication

This document describes the two authentication systems in Appleland: **user authentication** (Supabase JWT) and **admin authentication** (session cookies with Argon2 password verification).

---

## 1. User Authentication (Supabase JWT)

### Overview

User-facing routes (`/api/**`) authenticate via a Supabase-issued JSON Web Token (JWT) sent in the `Authorization: Bearer <token>` header. The token is verified against Supabase's public JSON Web Key Set (JWKS) using ECDSA (ES256).

### Flow

1. The client obtains a JWT from Supabase (via Supabase Auth sign-in).
2. The client sends the token in the `Authorization: Bearer <token>` header on every `/api/**` request.
3. `user_auth_middleware` (`src/middleware.rs`) extracts the token and calls `SupabaseAuth::verify()`.
4. `SupabaseAuth` (`src/auth/supabase.rs`) decodes the JWT header to find the key ID (`kid`), fetches the JWKS from `https://{project_ref}.supabase.co/auth/v1/.well-known/jwks.json`, and validates the signature using the matching EC public key.
5. If valid, the middleware checks that `claims.role == "authenticated"` and inserts the `Claims` into request extensions.
6. `user_row_middleware` then resolves `claims.sub` (the Supabase UUID) to an internal user UUID via a database lookup, inserting a `CurrentUser` struct into request extensions.

### What Is JWKS

JWKS (JSON Web Key Set) is a public endpoint exposed by Supabase at `https://{project_ref}.supabase.co/auth/v1/.well-known/jwks.json`. It contains the public keys (EC keys in this case) that correspond to the private keys Supabase uses to sign JWTs. Each key has a `kid` (key ID) and `x`/`y` coordinates for the elliptic curve point.

When a JWT arrives, the `kid` in its header is used to select the matching public key from the JWKS. The server then verifies the token's ES256 signature against that public key. This is standard public-key cryptography: Supabase signs with its private key, and anyone with the public key can verify the signature without needing to call Supabase.

### Why Cache JWKS

Without caching, every authenticated request would trigger an HTTP call to Supabase's JWKS endpoint. That adds latency (a network round-trip per request) and creates a hard dependency on Supabase being reachable — if the JWKS endpoint is down, no one can log in. Caching the JWKS in memory eliminates both problems: verification is purely local, and a temporary Supabase outage doesn't block authentication.

### How the Cache Works

The cache is an `Arc<RwLock<Option<CachedJwks>>>` where `CachedJwks` holds the parsed JWKS and an `Instant` of when it was fetched. The TTL is 10 minutes.

**Read path** (`get_jwks`):
1. Acquire a read lock. If the cache is populated and `fetched_at.elapsed() < TTL`, return the cached JWKS immediately.
2. If stale or empty, acquire the inflight mutex. If another task is already fetching, subscribe to its `broadcast::Sender` and wait for the result — this is **inflight deduplication**: concurrent requests share one upstream fetch.
3. If no fetch is in progress, create a new `broadcast::channel`, store the sender in the inflight slot, release the mutex, and perform the HTTP GET.
4. On success, populate the cache (write lock), clear the inflight slot, and broadcast the result to any waiting subscribers.

This ensures that even under high concurrency, at most one request per TTL window hits Supabase's JWKS endpoint.

### Error Handling

| Condition                                 | HTTP Status                |
| ----------------------------------------- | -------------------------- |
| Missing `Authorization` header            | 401 Unauthorized           |
| Expired token                             | 401 Unauthorized           |
| Invalid signature or malformed token      | 401 Unauthorized           |
| Valid token but `role != "authenticated"` | 403 Forbidden              |
| JWKS fetch failure                        | 500 Internal Server Error  |
| User not found in local DB                | 403 Account Setup Required |

### Justification

- **Stateless verification**: JWTs are verified locally against JWKS — no per-request call to Supabase. This keeps latency low and avoids a network dependency on every request.
- **JWKS caching with inflight deduplication**: Avoids hammering Supabase's JWKS endpoint while ensuring concurrent requests don't trigger redundant fetches.
- **Role check**: Supabase can issue tokens for different roles (e.g., `service_role`). The explicit `role == "authenticated"` check ensures only end-user tokens are accepted on user routes.
- **Separate user resolution middleware**: Decouples Supabase identity from the internal user model, allowing the internal user table to store application-specific data (e.g., purchase history) keyed by a stable internal UUID.

---

## 2. Admin Authentication (Session Cookies)

### Overview

Admin routes (`/admin/**`) authenticate via a session cookie. The admin submits a username and password; on success, the server creates a server-side session stored in PostgreSQL and returns a session cookie.

### Flow

1. `POST /admin/login` receives `{ username, password }`.
2. `admin_login_handler` (`src/auth/admin/handler.rs`) looks up the admin by username.
3. The stored Argon2 password hash is parsed and the submitted password is verified.
4. On success, the admin's `id` and `role` are stored in the session, and `session.cycle_id()` is called to rotate the session ID (preventing session fixation).
5. The `SessionManagerLayer` (`src/main.rs`) sets a cookie named `id` with `Secure`, `SameSite=Lax`, and 24-hour inactivity expiry.
6. Subsequent requests include the cookie; `admin_auth_middleware` checks for a valid session with a `role` key.
7. `POST /admin/logout` calls `session.delete()`, destroying the server-side session.

### How Session Cookies Work

**Login**: The admin submits credentials. On success, the server creates a session record in PostgreSQL containing the admin's `id` and `role`. The `SessionManagerLayer` then emits a `Set-Cookie` header with a cookie named `id` whose value is the session ID (a random, unguessable token). The cookie attributes are:

- `Secure` — only sent over HTTPS, preventing interception on plain HTTP.
- `HttpOnly` — set by `tower_sessions`, prevents JavaScript from reading the cookie (mitigates XSS-based session theft).
- `SameSite=Lax` — sent on same-site requests and top-level navigations, blocked on cross-site POST. Sufficient for cross-origin frontend/backend deployments where the frontend calls the API with `credentials: 'include'`.

**Subsequent requests**: The browser automatically includes the `id` cookie. The `SessionManagerLayer` reads the cookie, looks up the session in PostgreSQL, and deserializes the stored `id` and `role` values. `admin_auth_middleware` then checks that a `role` exists in the session.

**Expiry**: Sessions use `Expiry::OnInactivity(Duration::hours(24))` — the session expires after 24 hours of inactivity. Each request resets the inactivity timer. A background task (`continuously_delete_expired`) removes expired sessions from PostgreSQL at the interval specified by `SESSION_DELETE_INTERVAL` (default `8h`).

**Logout**: `POST /admin/logout` calls `session.delete()`, which removes the session from PostgreSQL and expires the cookie.

### Why Server-Side Sessions (Not JWTs)

- **Immediate revocation**: A session can be destroyed instantly (logout, password change, admin removal) without waiting for a token to expire. With JWTs, the token remains valid until expiry unless a denylist is maintained.
- **No token payload bloat**: The session stores only `id` and `role`. A JWT would need to carry all claims in every request.
- **Simpler key management**: No need to rotate signing keys or manage JWKS for the admin side. The session ID is just a random token; the authoritative state is in PostgreSQL.
- **Natural fit for cookie-based auth**: Sessions are designed to work with cookies, which provide built-in security attributes (`HttpOnly`, `Secure`, `SameSite`) that JWTs in `localStorage` lack.

### Password Hashing

- **Algorithm**: Argon2 (default parameters via `Argon2::default()`).
- **Storage format**: PHC string format (e.g., `$argon2id$v=19$m=...,t=...,p=...$<salt>$<hash>`).
- **Verification**: `PasswordHash::new(&stored_hash)` parses the PHC string, then `Argon2::default().verify_password()` checks the plaintext.

### Timing Attack Prevention

When a username is not found, the handler hashes a dummy password (`b"dummy"`) with a fixed 16-byte zero salt. This ensures the response time is similar to a real login attempt, preventing user enumeration via timing analysis.

### Session Fixation Prevention

After successful authentication, `session.cycle_id()` rotates the session ID. This ensures an attacker who pre-set a session ID on a victim's browser cannot use that ID after the victim logs in.

### Admin Roles

Admins have a role stored in the database (PostgreSQL enum `admin_role`):

| Role               | Description                            |
| ------------------ | -------------------------------------- |
| `sale_admin`       | Can manage sales and inventory         |
| `playground_admin` | Can manage playground-related features |
| `super_admin`      | Full access                            |

The role is stored in the session at login time and can be checked by handlers or extractors (e.g., `RequiresRole<T>`).

### Justification

- **Server-side sessions**: Unlike JWTs, sessions can be instantly invalidated (e.g., on logout or password change) without waiting for token expiry. This is important for admin access where immediate revocation matters.
- **Argon2**: Memory-hard password hashing resistant to GPU/ASIC attacks. The default parameters (as of `argon2` crate) provide a good balance of security and performance.
- **Session ID rotation**: Prevents session fixation attacks where an attacker sets a known session ID before the victim authenticates.
- **Timing attack mitigation**: Dummy hashing on unknown usernames prevents attackers from determining whether a username exists by measuring response times.
- **`SameSite=Lax`**: Sufficient for cross-origin frontend/backend deployments (the frontend calls the API with `credentials: 'include'`). `SameSite=None` is not needed because the app is not embedded in a third-party iframe.

---

## 3. Middleware Stack

### User Routes (`/api/**`)

```
Request
  -> Rate limiting (tower_governor)
  -> CORS
  -> Timeout (10s)
  -> Request ID
  -> user_auth_middleware (JWT verification)
  -> user_row_middleware (resolve internal user)
  -> Handler
```

### Admin Routes (`/admin/**`)

```
Request
  -> Rate limiting (tower_governor)
  -> CORS
  -> Timeout (10s)
  -> Request ID
  -> SessionManagerLayer (parse session cookie)
  -> admin_auth_middleware (check session + role)
  -> Handler
```

---

## 4. Configuration

| Environment Variable      | Purpose                                                 | Default |
| ------------------------- | ------------------------------------------------------- | ------- |
| `SUPABASE_REF`            | Supabase project reference (used to construct JWKS URL) | —       |
| `ALLOWED_ORIGINS`         | Comma-separated list of allowed CORS origins            | —       |
| `SESSION_DELETE_INTERVAL` | Interval for expired session cleanup                    | `8h`    |
| `BURST_SIZE`              | Rate limit burst size                                   | `15`    |
| `PER_SEC`                 | Rate limit requests per second                          | `10`    |

---

## 5. Security Considerations

- **CSRF**: `SameSite=Lax` provides baseline CSRF protection for admin routes. User routes use Bearer tokens (not cookies), so they are not vulnerable to CSRF.
- **XSS**: The session cookie is `HttpOnly` (set by `tower_sessions`), preventing JavaScript access.
- **Password storage**: Argon2 with default parameters. No plaintext passwords are stored.
- **Session expiry**: 24-hour inactivity timeout. Expired sessions are cleaned up by a background task.
- **Rate limiting**: Applied globally to both user and admin routes to mitigate brute-force attacks.
