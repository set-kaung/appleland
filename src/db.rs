use sqlx::{
    Postgres,
    pool::Pool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub async fn connect_db(db_url: String) -> Result<Pool<Postgres>, sqlx::Error> {
    let mut connect_options: PgConnectOptions = db_url.parse()?;
    connect_options = connect_options.options([("statement_timeout", "10000")]);
    PgPoolOptions::new().connect_with(connect_options).await
}
