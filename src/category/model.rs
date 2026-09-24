use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize, ToSchema)]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub updated_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub is_deleted: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub deleted_by: Option<Uuid>,
}

#[derive(FromRow, Serialize, Deserialize, ToSchema)]
pub struct InsertCategory {
    pub name: String,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateCategoryPayload {
    pub name: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct GetCategoriesQuery {
    #[serde(default)]
    pub is_deleted_shown: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AdminCategory {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub updated_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub is_deleted: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub deleted_by: Option<Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateCategoryPayload {
    pub name: String,
    pub updated_by: Uuid,
}

pub struct UpdateCategory {
    pub id: Uuid,
    pub name: String,
    pub updated_by: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteCategoryResponse {
    pub id: Uuid,
    pub is_deleted: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub deleted_by: Option<Uuid>,
}

impl From<CategoryRow> for AdminCategory {
    fn from(value: CategoryRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_by: value.created_by,
            created_at: value.created_at,
            updated_by: value.updated_by,
            updated_at: value.updated_at,
            is_deleted: value.is_deleted,
            deleted_at: value.deleted_at,
            deleted_by: value.deleted_by,
        }
    }
}
