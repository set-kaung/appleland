use crate::auth::{AuthError, AuthProvider, Claims};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, errors::ErrorKind};
use jsonwebtoken::{Header, decode_header};
use reqwest;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone, Deserialize)]
pub struct Jwk {
    pub kid: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
struct CachedJwks {
    jwks: Jwks,
    fetched_at: Instant,
}

pub struct SupabaseAuth {
    jwks_url: String,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    cache_ttl: Duration,
    http: reqwest::Client,
}

impl SupabaseAuth {
    pub fn new(project_ref: String) -> Self {
        Self {
            jwks_url: format!(
                "https://{}.supabase.co/auth/v1/.well-known/jwks.json",
                project_ref
            ),
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(600),
            http: reqwest::Client::new(),
        }
    }

    async fn get_jwks(&self) -> Result<Jwks, AuthError> {
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache
                && cached.fetched_at.elapsed() < self.cache_ttl
            {
                return Ok(cached.jwks.clone());
            }
        }

        let jwks = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::Provider(e.to_string()))?
            .json::<Jwks>()
            .await
            .map_err(|e| AuthError::Provider(e.to_string()))?;

        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedJwks {
                jwks: jwks.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(jwks)
    }
}

#[async_trait]
impl AuthProvider for SupabaseAuth {
    async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let header: Header = match decode_header(token) {
            Ok(h) => h,
            Err(err) => {
                println!("error decoding jwks header: {}", err);
                return Err(AuthError::InvalidToken);
            }
        };

        let jwks = self.get_jwks().await?;

        let jwk = match &header.kid {
            Some(kid) => jwks
                .keys
                .iter()
                .find(|k| k.kid.as_deref() == Some(kid.as_str()))
                .ok_or(AuthError::InvalidToken)?,
            None => jwks.keys.first().ok_or(AuthError::InvalidToken)?,
        };

        let decoding_key = DecodingKey::from_ec_components(
            jwk.x.as_deref().ok_or(AuthError::InvalidToken)?,
            jwk.y.as_deref().ok_or(AuthError::InvalidToken)?,
        )
        .map_err(|_| AuthError::InvalidToken)?;

        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&["authenticated"]);

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(data) => Ok(data.claims),
            Err(e) => match e.kind() {
                ErrorKind::ExpiredSignature => Err(AuthError::Expired),
                _ => {
                    println!("decode error: {:?}", e); // add this
                    Err(AuthError::InvalidToken)
                }
            },
        }
    }
}
