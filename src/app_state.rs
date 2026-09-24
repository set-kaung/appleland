use sqlx::PgPool;

use crate::{auth::AuthProvider, payment::PaymentProvider};
use std::sync::Arc;

#[derive(Clone)]
pub struct StorageConfig {
    pub supabase_storage_ref: String,
    pub supabase_storage_url: String,
    pub supabase_storage_secret_key: String,
    pub image_bucket_name: String,
}

#[derive(Clone)]
pub struct AiConfig {
    pub gemini_api_key: String,
    pub daily_limit: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: Arc<dyn AuthProvider>,
    pub storage: StorageConfig,
    pub payment_provider: Arc<dyn PaymentProvider>,
    pub demo_mode: bool,
    pub ai: AiConfig,
}

impl AppState {
    pub fn new(
        auth: Arc<dyn AuthProvider>,
        db: PgPool,
        storage: StorageConfig,
        payment_provider: Arc<dyn PaymentProvider>,
        demo_mode: bool,
        ai: AiConfig,
    ) -> Self {
        Self {
            db,
            auth,
            storage,
            payment_provider,
            demo_mode,
            ai,
        }
    }
}
