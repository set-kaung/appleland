use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize, ToSchema)]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_by: Uuid,
    pub updated_at: OffsetDateTime,
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

#[derive(FromRow, Serialize, Deserialize, ToSchema)]
pub struct AdminCategory {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_by: Uuid,
    pub updated_at: OffsetDateTime,
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
        }
    }
}
