use axum::Router;
use axum::routing::{get};
use crate::app_state::AppState;
use crate::category::handler::{create_category, get_all_categories};

pub mod model;
pub mod store;
pub mod handler;



pub fn route_admin() -> Router<AppState>{
    axum::Router::new().route(
        "/", get(get_all_categories).post(create_category)
    )
}