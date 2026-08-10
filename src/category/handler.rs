use super::store;
use crate::ApiResult;
use crate::app_error::APIError;
use crate::app_extractor::ValidatedJson;
use crate::app_state;
use crate::app_state::AppState;
use crate::auth::password::session_admin_id;
use crate::category::model::{
    AdminCategory, CreateCategoryPayload, InsertCategory, UpdateCategory, UpdateCategoryPayload,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/admin/categories",
    description = "Create a new product category.",
    request_body = CreateCategoryPayload,
    responses(
        (status = 201, description = "Category created", body = AdminCategory),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Categories"
)]
pub async fn create_category(
    State(state): State<app_state::AppState>,
    session: Session,
    ValidatedJson(payload): ValidatedJson<CreateCategoryPayload>,
) -> Result<(StatusCode, Json<AdminCategory>), APIError> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to start transc: {e}");
        APIError::InternalErr
    })?;
    let admin_id = session_admin_id(session).await?;

    let insert_category = InsertCategory {
        name: payload.name,
        created_by: admin_id,
        updated_by: admin_id,
    };

    let category = store::create_category(&mut tx, insert_category)
        .await?
        .into();
    tx.commit().await.map_err(|e| {
        error!("failed to commit transc: {e}");
        APIError::InternalErr
    })?;
    Ok((StatusCode::CREATED, Json(category)))
}

#[utoipa::path(
    get,
    path = "/admin/categories",
    description = "Fetch every product category.",
    responses(
        (status = 200, description = "List of all categories", body = Vec<AdminCategory>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Categories"
)]
pub async fn get_all_categories(
    State(s): State<AppState>,
) -> Result<Json<Vec<AdminCategory>>, APIError> {
    let categories = store::get_all_categories(&s.db)
        .await?
        .into_iter()
        .map(|e| e.into())
        .collect();

    Ok(Json(categories))
}

#[utoipa::path(
    put,
    path = "/admin/categories/{id}",
    description = "Update a product category name.",
    request_body = UpdateCategoryPayload,
    responses(
        (status = 200, description = "Category updated", body = AdminCategory),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Categories"
)]
pub async fn update_category(
    State(s): State<AppState>,
    session: Session,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryPayload>,
) -> ApiResult<AdminCategory> {
    let admin_id = session_admin_id(session).await?;
    let update_category = UpdateCategory {
        id,
        name: payload.name,
        updated_by: admin_id,
    };
    let result = store::update_category(&s.db, update_category).await?.into();
    Ok(Json(result))
}
