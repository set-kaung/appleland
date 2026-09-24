// #![warn(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![warn(
    clippy::dbg_macro,
    clippy::todo,
    clippy::unimplemented,
    clippy::redundant_clone,
    clippy::needless_collect,
    clippy::inefficient_to_string,
    clippy::large_futures,
    clippy::await_holding_lock
)]
// #![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

use axum::Json;
use axum::http::StatusCode;

pub mod admin_settings;
pub mod api_doc;
pub mod app_error;
pub mod app_extractor;
pub mod app_state;
pub mod auth;
pub mod category;
pub mod config;
pub mod db;
pub mod image_storage;
pub mod inventory;
pub mod middleware;
pub mod orders;
pub mod pagination;
pub mod payment;
pub mod playground;
pub mod product;
pub mod routes;
pub mod sales;
pub mod staff;
pub mod users;

pub type ApiResult<T> = Result<Json<T>, app_error::APIError>;
pub type ApiResultWithStatusCode<T> = Result<(StatusCode, Json<T>), app_error::APIError>;
pub type PaginatedApiResult<T> =
    Result<Json<pagination::PaginatedResponse<T>>, app_error::APIError>;
