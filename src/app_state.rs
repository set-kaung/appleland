use sqlx::PgPool;

use crate::{auth::AuthProvider, payment::PaymentProvider};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub supabase_ref: String,
    pub supabase_secret_key: String,
    pub image_bucket_name: String,
    pub payment_provider: Arc<dyn PaymentProvider>,
}

impl AppState {
    pub fn new(
        auth: Arc<dyn AuthProvider>,
        db: PgPool,
        supabase_ref: String,
        supabase_secret_key: String,
        image_bucket_name: String,
        payment_provider: Arc<dyn PaymentProvider>,
    ) -> Self {
        Self {
            db,
            auth,
            supabase_ref,
            supabase_secret_key,
            image_bucket_name,
            payment_provider,
        }
    }
}
