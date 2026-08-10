use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Database error occurred: {0}")]
    Database(sqlx::Error),

    #[error("Query affected {0} rows instead of the expected 1. Rolled back.")]
    UnexpectedRowsAffected(u64),

    #[error("record not found")]
    RecordNotFound,

    #[error("record with {0} already exists")]
    UniqueKeyViolated(String),

    #[error("required field missing")]
    NullField(String),

    #[error("insufficient inventory for product {0}: requested {1}, available {2}")]
    InsufficientInventory(uuid::Uuid, i32, i32),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to load config: {0}")]
    ConfigErr(#[from] envy::Error),

    #[error("failed to connect to database: {0}")]
    DatabaseConnectionErr(#[from] sqlx::Error),

    #[error("session store error")]
    SessionStoreErr,

    #[error("doc gen error")]
    DocumentGenerationError(String),

    #[error("general error")]
    GeneralErr,

    #[error("starup setup error")]
    StartupErr(String),
}
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error("invalid credentials")]
    AdminInvalidCreds,

    #[error("internal server error")]
    InternalErr,

    #[error("forbidden")]
    Forbidden,

    #[error("bad request")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found")]
    NotFound,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        match self {
            APIError::InternalErr => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            APIError::AdminInvalidCreds => {
                (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
            }
            APIError::Forbidden => (StatusCode::FORBIDDEN, "forbidden").into_response(),
            APIError::BadRequest(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            APIError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
            APIError::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::Database(db) => {
                if let Some(pg) = db.try_downcast_ref::<sqlx::postgres::PgDatabaseError>() {
                    return match pg.code() {
                        "23505" => StoreError::UniqueKeyViolated(
                            pg.constraint()
                                .map_or("missing field".to_string(), |s| s.to_string()),
                        ),
                        "23502" => StoreError::NullField(
                            pg.constraint()
                                .map_or("missing field".to_string(), |s| s.to_string()),
                        ),
                        _ => StoreError::Database(e),
                    };
                }
                StoreError::Database(e)
            }

            sqlx::Error::RowNotFound => StoreError::RecordNotFound,

            _ => StoreError::Database(e),
        }
    }
}

impl From<StoreError> for APIError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::RecordNotFound => APIError::NotFound,
            StoreError::UnexpectedRowsAffected(n) => {
                tracing::error!(rows = n, "unexpected row count");
                APIError::InternalErr
            }
            StoreError::Database(e) => {
                tracing::error!(?e, "database error");
                APIError::InternalErr
            }
            StoreError::UniqueKeyViolated(e) => APIError::BadRequest(format!("{e} must be unique")),
            StoreError::NullField(e) => APIError::BadRequest(format!("missing field: {e}")),
            StoreError::InsufficientInventory(id, requested, available) => {
                APIError::BadRequest(format!(
                    "insufficient inventory for product {id}: requested {requested}, available {available}"
                ))
            }
            StoreError::InvalidInput(e) => APIError::BadRequest(e),
        }
    }
}
