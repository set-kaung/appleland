#[derive(Debug, thiserror::Error)] // Using the popular `thiserror` crate
pub enum StoreError {
    #[error("Database error occurred: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Safety trigger: Query attempted to delete {0} rows instead of 1. Rolled back.")]
    SafetyTrigger(u64),
}
