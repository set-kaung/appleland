use sqlx::PgPool;

use crate::auth::AuthProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
}
impl AppState {
    pub fn new(auth: Arc<dyn AuthProvider>, db: PgPool) -> Self {
        Self { auth, db }
    }
}
