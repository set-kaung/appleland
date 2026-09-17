use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
mod supabase;
pub use supabase::SupabaseAuth;
pub mod password;
pub use password::{AdminRole, admin_login_handler, admin_logout_handler};

#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    async fn verify(&self, token: &str) -> Result<Claims, AuthError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    pub aud: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CurrentUser {
    pub id: Uuid,
    pub supabase_id: Uuid,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("invalid token")]
    InvalidToken,
    #[error("expired token")]
    Expired,
    #[error("provider error: {0}")]
    Provider(String),
}
