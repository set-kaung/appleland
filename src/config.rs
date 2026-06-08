use crate::app_error::AppError;
use axum::http::HeaderValue;
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
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "port: {}\nsupabase_ref: {}\ndatabase_host: {}\nallowed_origins: {}\nburst_size: {}\nper_sec: {}\n",
            self.port,
            self.supabase_ref,
            database_host(&self.database_url),
            self.allowed_origins
                .iter()
                .map(|x| x.to_str().unwrap().to_string())
                .collect::<Vec<String>>()
                .join(" "),
            self.burst_size,
            self.per_sec
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

pub fn load_config() -> Result<Config, AppError> {
    match std::env::var("BURST_SIZE") {
        Ok(v) => println!("Raw BURST_SIZE from env: {:?}", v.as_bytes()),
        Err(e) => println!("BURST_SIZE not found: {e}"),
    }
    match envy::from_env::<Config>().map_err(AppError::ConfigErr) {
        Ok(conf) => {
            println!("Configurations\n==============\n{}", conf);
            Ok(conf)
        }
        Err(err) => Err(err),
    }
}
