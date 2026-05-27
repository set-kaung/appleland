use sqlx::PgPool;

use crate::auth::AuthProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    auth: Arc<dyn AuthProvider>,
    pub db: PgPool,
}

impl AppState {
    pub fn new(auth: Arc<dyn AuthProvider>, db: PgPool) -> Self {
        AppState { auth, db }
    }

    pub fn auth_arc(&self) -> Arc<dyn AuthProvider> {
        Arc::clone(&self.auth)
    }
}
