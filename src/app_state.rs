use sqlx::PgPool;

use crate::{auth::AuthProvider, payment::PaymentProvider};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub uploadthing_token: String,
    pub payment_provider: Arc<dyn PaymentProvider>,
}

impl AppState {
    pub fn new(
        auth: Arc<dyn AuthProvider>,
        db: PgPool,
        uploadthing_token: String,
        payment_provider: Arc<dyn PaymentProvider>,
    ) -> Self {
        Self {
            db,
            auth,
            uploadthing_token,
            payment_provider,
        }
    }
}
