use axum::extract::State;
use axum::Json;
use tower_sessions::Session;
use tracing::error;
use crate::{app_extractor, app_state};
use crate::app_error::APIError;
use crate::auth::password::session_admin_id;
use crate::category::model::{AdminCategory, CreateCategoryPayload, InsertCategory};
use super::store;


#[utoipa::path(
    post,
    path = "/admin/categories",
    request_body = CreateCategoryPayload,
    responses(
        (status = 201, body = AdminCategory),
        (status = 400),
        (status = 401),
        (status = 409),
        (status = 500)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Categories"
)]
pub async fn create_category(
    State(state): State<app_state::AppState>,
    session: Session,
    app_extractor::ValidatedJson(payload): app_extractor::ValidatedJson<CreateCategoryPayload>)
    -> Result<Json<AdminCategory>, APIError>
{
    let mut tx = state.db.begin().await.map_err(|e|
        {
            error!("failed to start transc: {e}");
            APIError::InternalErr
        }
    )?;
    let admin_id = session_admin_id(session).await?;

    let insert_category = InsertCategory{
        name: payload.name,
        created_by: admin_id,
        updated_by: admin_id,
    };

    let category = store::create_category(&mut tx,insert_category).await?.into();
    tx.commit().await.map_err(|e|
        {
            error!("failed to commit transc: {e}");
            APIError::InternalErr
        }
    )?;
    Ok(Json(category))
}
