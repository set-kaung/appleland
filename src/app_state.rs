use sqlx::PgPool;

use crate::auth::AuthProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub uploadthing_token: String,
}

impl AppState {
    pub fn new(auth: Arc<dyn AuthProvider>, db: PgPool, uploadthing_token: String) -> Self {
        Self {
            db,
            auth,
            uploadthing_token,
        }
    }
}
