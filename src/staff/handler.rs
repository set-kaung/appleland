use crate::ApiResult;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::staff::model::AdminStaff;
use crate::staff::store;
use axum::Json;
use axum::extract::State;

#[utoipa::path(
    get,
    path = "/admin/staff",
    description = "Get all staff",
    responses(
        (status = 200, description = "All staff", body = Vec<AdminStaff>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Staff"
)]
pub async fn get_all_admin_staff(
    State(state): State<AppState>,
    _require_role: RequiresRole<SuperAdminRole>,
) -> ApiResult<Vec<AdminStaff>> {
    let result = store::get_admins(&state.db).await?;
    Ok(Json(result))
}
