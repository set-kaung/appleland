use crate::app_state::AppState;
use crate::staff::handler::get_all_admin_staff;
use axum::Router;
use axum::routing::get;

pub mod handler;
pub mod model;
pub mod store;

pub fn route_admin() -> Router<AppState> {
    Router::new().route("/", get(get_all_admin_staff))
}
