use axum::Router;
use axum::routing::post;
use crate::app_state::AppState;
use crate::category::handler::create_category;

pub mod model;
pub mod store;
pub mod handler;



pub fn route_admin() -> Router<AppState>{
    axum::Router::new().route(
        "/", post(create_category)
    )
}