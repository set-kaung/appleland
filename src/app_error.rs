use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Database error occurred: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Safety trigger: Query attempted to delete {0} rows instead of 1. Rolled back.")]
    SafetyTrigger(u64),

    #[error("record not found")]
    NoRows,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to load fonig: {0}")]
    ConfigErr(#[from] envy::Error),

    #[error("failed to connect to database: {0}")]
    DatabaseConnectionErr(#[from] sqlx::Error),

    #[error("session store error")]
    SessionStoreErr,

    #[error("general error")]
    GeneralErr,
}
#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error("invalid credentials")]
    AdminInvalidCreds,

    #[error("internal server error")]
    InternalErr,

    #[error("forbidden")]
    Forbidden,
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
        }
    }
}
