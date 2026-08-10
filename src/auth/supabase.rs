use crate::auth::{AuthError, AuthProvider, Claims};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, errors::ErrorKind};
use jsonwebtoken::{Header, decode_header};
use reqwest;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, broadcast};

pub type InflightJwks = Arc<Mutex<Option<broadcast::Sender<Result<Jwks, AuthError>>>>>;

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
    inflight: InflightJwks,
}

impl SupabaseAuth {
    pub fn new(project_ref: &str) -> Self {
        Self {
            jwks_url: format!("https://{project_ref}.supabase.co/auth/v1/.well-known/jwks.json"),
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_mins(10),
            http: reqwest::Client::new(),
            inflight: Arc::new(Mutex::new(None)),
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

        let mut inflight = self.inflight.lock().await;

        if let Some(tx) = inflight.as_ref() {
            let mut rx = tx.subscribe();
            drop(inflight);
            match rx.recv().await {
                Ok(result) => return result,
                Err(_) => {
                    return Err(AuthError::Provider("JWKS fetch aborted".to_string()));
                }
            }
        }

        let (tx, _) = broadcast::channel(1);
        *inflight = Some(tx.clone());
        drop(inflight);

        let result = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::Provider(e.to_string()))?
            .json::<Jwks>()
            .await
            .map_err(|e| AuthError::Provider(e.to_string()));

        if let Ok(ref jwks) = result {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedJwks {
                jwks: jwks.clone(),
                fetched_at: Instant::now(),
            });
        }

        let _ = tx.send(result.clone());

        let mut inflight = self.inflight.lock().await;
        *inflight = None;

        result
    }
}

#[async_trait]
impl AuthProvider for SupabaseAuth {
    async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let header: Header = match decode_header(token) {
            Ok(h) => h,
            Err(err) => {
                println!("error decoding jwks header: {err}");
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
            Err(e) => {
                if let ErrorKind::ExpiredSignature = e.kind() {
                    Err(AuthError::Expired)
                } else {
                    println!("decode error: {e:?}");
                    Err(AuthError::InvalidToken)
                }
            }
        }
    }
}
