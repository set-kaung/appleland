#[derive(Debug, thiserror::Error)] // Using the popular `thiserror` crate
pub enum StoreError {
    #[error("Database error occurred: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Safety trigger: Query attempted to delete {0} rows instead of 1. Rolled back.")]
    SafetyTrigger(u64),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to load fonig: {0}")]
    ConfigErr(#[from] envy::Error),

    #[error("failed to connect to database: {0}")]
    DatabaseConnectionErr(#[from] sqlx::Error),
}
