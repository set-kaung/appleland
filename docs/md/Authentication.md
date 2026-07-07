# Authentication

## Supabase User Login

Authenticate the user by sending a request to your Supabase project's `auth/v1/token` endpoint.

- Query parameter:
  - `grant_type=password`

- Required headers:
  - `apikey: <SUPABASE_ANON_KEY>` (or the appropriate API key)

- Request body:

  ```json
  {
    "email": "user@example.com",
    "password": "password"
  }
  ```

The response contains an `access_token`. Include this token in subsequent requests to the backend:

```http
Authorization: Bearer <access_token>
```

Refer to the Supabase Authentication documentation for additional details.

---

# Admin Login

Send a POST request to the admin login endpoint with the following JSON body:

```json
{
  "username": "string",
  "password": "string"
}
```

If authentication succeeds, the backend sets the required session cookie automatically.

For browser-based clients, no additional action is required. Ensure your HTTP client allows cookies to be stored and sent with subsequent requests (for example, use `credentials: "include"` when using the Fetch API if the frontend and backend are on different origins).

Subsequent admin requests will be authenticated using the session cookie.
