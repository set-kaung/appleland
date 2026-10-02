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
        .log_statements(tracing::log::LevelFilter::Trace)
        .application_name("appleland_api");
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

pub trait DBQueryFilter<DB: sqlx::Database> {
    fn apply_filter(&self, query_builder: &mut sqlx::QueryBuilder<DB>);

    /// Add an ORDER BY clause. The default is no sorting.
    fn apply_sort(&self, _query_builder: &mut sqlx::QueryBuilder<DB>) {}
}

pub fn escape_like(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        if matches!(ch, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
