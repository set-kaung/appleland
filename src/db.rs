use sqlx::ConnectOptions;
use sqlx::{
    Postgres,
    pool::Pool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub async fn connect_db(db_url: String) -> Result<Pool<Postgres>, sqlx::Error> {
    let mut connect_options: PgConnectOptions = db_url.parse()?;
    connect_options = connect_options
        .options([("statement_timeout", "10000")])
        .log_statements(tracing::log::LevelFilter::Trace);
    match PgPoolOptions::new().connect_with(connect_options).await {
        Ok(pool) => {
            let options = pool.connect_options();
            tracing::info!(
                "Connected to: {}:{}",
                options.get_host(),
                options.get_port()
            );
            Ok(pool)
        }
        Err(err) => Err(err),
    }
}
