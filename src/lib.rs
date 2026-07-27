// #![warn(clippy::pedantic)]
// #![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]
// #![deny(clippy::unwrap_used, clippy::expect_used)]

use axum::Json;
use reqwest::StatusCode;

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
pub type ApiResultWithStatusCode<T> = Result<(StatusCode, Json<T>), app_error::APIError>;
pub type PaginatedApiResult<T> =
    Result<Json<pagination::PaginatedResponse<T>>, app_error::APIError>;
