use crate::app_error::AppError;
use axum::http::HeaderValue;
use clap::Parser;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub supabase_auth_ref: String,
    pub database_url: String,
    pub supabase_storage_ref: String,
    pub supabase_storage_url: String,
    pub supabase_storage_secret_key: String,
    #[serde(default = "default_image_bucket_name")]
    pub image_bucket_name: String,
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub allowed_origins: Vec<HeaderValue>,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
    #[serde(default = "default_per_sec")]
    pub per_sec: u64,

    #[serde(default)]
    pub demo_mode: bool,

    pub gemini_api_key: String,

    #[serde(default = "default_gemini_model")]
    pub gemini_model: String,

    #[serde(default = "default_ai_chat_daily_limit")]
    pub ai_chat_daily_limit: u32,

    #[serde(default = "default_expiry_interval")]
    pub session_delete_interval: String,

    #[serde(default = "default_playground_cleanup_interval")]
    pub playground_token_cleanup_interval: String,

    #[serde(default = "default_nonce_cleanup_interval")]
    pub webhook_nonce_cleanup_interval: String,

    #[serde(default)]
    pub rust_log: Option<String>,

    pub mmpay_webhook_secret: String,
    pub mmpay_publishable_key: String,
    #[serde(default = "default_true")]
    pub mmpay_sandbox: bool,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        indoc::writedoc!(
            f,
            "
                port: {}
                supabase_auth_ref: {}
                supabase_storage_ref: {}
                supabase_storage_url: {}
                database_host: {}
                allowed_origins: {}
                burst_size: {}
                per_sec: {}
                demo_mode: {}
                gemini_model: {}
                ai_chat_daily_limit: {}
                session_delete_interval: {}
                playground_token_cleanup_interval: {}
                webhook_nonce_cleanup_interval: {}
                rust_log: {}",
            self.port,
            self.supabase_auth_ref,
            self.supabase_storage_ref,
            self.supabase_storage_url,
            database_host(&self.database_url),
            self.allowed_origins
                .iter()
                .map(|x| x.to_str().unwrap_or("<invalid>").to_string())
                .collect::<Vec<String>>()
                .join(" "),
            self.burst_size,
            self.per_sec,
            self.demo_mode,
            self.gemini_model,
            self.ai_chat_daily_limit,
            self.session_delete_interval,
            self.playground_token_cleanup_interval,
            self.webhook_nonce_cleanup_interval,
            self.rust_log.clone().unwrap_or("NONE".to_string()),
        )
    }
}

fn database_host(database_url: &str) -> String {
    match database_url.parse::<PgConnectOptions>() {
        Ok(options) => options.get_host().to_string(),
        Err(_) => "<invalid_database_url>".to_string(),
    }
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<HeaderValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.split(',')
        .map(|s| s.parse::<HeaderValue>().map_err(serde::de::Error::custom))
        .collect::<Result<Vec<_>, _>>()
}

fn default_port() -> u16 {
    let port = 8080;
    tracing::info!("using default port: {}", port);
    port
}

fn default_per_sec() -> u64 {
    10
}

fn default_burst_size() -> u32 {
    15
}

fn default_expiry_interval() -> String {
    "1m".to_string()
}

fn default_playground_cleanup_interval() -> String {
    "30m".to_string()
}

fn default_nonce_cleanup_interval() -> String {
    "3h".to_string()
}

fn default_image_bucket_name() -> String {
    "appleland-product-images".to_string()
}

fn default_true() -> bool {
    true
}

fn default_ai_chat_daily_limit() -> u32 {
    20
}

fn default_gemini_model() -> String {
    "gemini-2.5-flash".to_string()
}

pub fn load_config() -> Result<Config, AppError> {
    match envy::from_env::<Config>().map_err(AppError::ConfigErr) {
        Ok(conf) => {
            tracing::info!("Configurations\n==============\n{}", conf);
            Ok(conf)
        }
        Err(err) => Err(err),
    }
}

#[derive(Debug, Parser)]
pub struct Cli {
    /// if set, generate documentation only and exit, default is false
    #[arg(short, long, default_value_t = false)]
    pub gen_docs_only: bool,
}
