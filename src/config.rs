use crate::app_error::AppError;
use axum::http::HeaderValue;
use clap::Parser;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub supabase_ref: String,
    pub database_url: String,
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub allowed_origins: Vec<HeaderValue>,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
    #[serde(default = "default_per_sec")]
    pub per_sec: u64,

    #[serde(default = "default_expiry_interval")]
    pub session_delete_interval: String,

    #[serde(default)]
    pub rust_log: Option<String>,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        indoc::writedoc!(
            f,
            "
                port: {}
                supabase_ref: {}
                database_host: {}
                allowed_origins: {}
                burst_size: {}
                per_sec: {}
                session_delete_interval: {}
                rust_log: {}",
            self.port,
            self.supabase_ref,
            database_host(&self.database_url),
            self.allowed_origins
                .iter()
                .map(|x| x.to_str().unwrap().to_string())
                .collect::<Vec<String>>()
                .join(" "),
            self.burst_size,
            self.per_sec,
            self.session_delete_interval,
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
    println!("using default port: {}", port);
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

pub fn load_config() -> Result<Config, AppError> {
    match envy::from_env::<Config>().map_err(AppError::ConfigErr) {
        Ok(conf) => {
            println!("Configurations\n==============\n{}", conf);
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
