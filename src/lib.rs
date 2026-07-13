use axum::Json;

pub mod api_doc;

pub mod app_error;
pub mod app_extractor;
pub mod app_state;
pub mod auth;
pub mod category;
pub mod config;
pub mod db;
pub mod inventory;
pub mod middleware;
pub mod order;
pub mod pagination;
pub mod product;
pub mod sales;

pub type ApiResult<T> = Result<Json<T>, app_error::APIError>;
pub type PaginatedApiResult<T> =
    Result<Json<pagination::PaginatedResponse<T>>, app_error::APIError>;
