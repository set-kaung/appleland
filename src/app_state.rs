use sqlx::PgPool;

use crate::{auth::AuthProvider, payment::PaymentProvider};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub supabase_storage_ref: String,
    pub supabase_storage_url: String,
    pub supabase_storage_secret_key: String,
    pub image_bucket_name: String,
    pub payment_provider: Arc<dyn PaymentProvider>,
    pub demo_mode: bool,
}

impl AppState {
    pub fn new(
        auth: Arc<dyn AuthProvider>,
        db: PgPool,
        supabase_storage_ref: String,
        supabase_storage_url: String,
        supabase_storage_secret_key: String,
        image_bucket_name: String,
        payment_provider: Arc<dyn PaymentProvider>,
        demo_mode: bool,
    ) -> Self {
        Self {
            db,
            auth,
            supabase_storage_ref,
            supabase_storage_url,
            supabase_storage_secret_key,
            image_bucket_name,
            payment_provider,
            demo_mode,
        }
    }
}
