use crate::auth::admin::model::AdminRole;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct AdminStaff {
    pub id: Uuid,
    pub username: String,
    pub role: AdminRole,
}
