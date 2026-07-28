# API Response

`lib.rs` file contains the generic API reponse with type for both normal and paginated reponses.

```rust
pub type ApiResult<T> = Result<Json<T>, app_error::APIError>;
pub type PaginatedApiResult<T> =
    Result<Json<pagination::PaginatedResponse<T>>, app_error::APIError>;
```

The value from the store should implement From trait where convenient.
