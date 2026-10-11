use crate::admin_settings::model::SettingResponse;
use crate::admin_settings::service;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::ApiResult;
use axum::Json;
use axum::extract::State;

#[utoipa::path(
    get,
    path = "/admin/settings",
    description = "List all store settings ordered by key, ascending.",
    responses(
        (status = 200, description = "All settings", body = Vec<SettingResponse>),
        (status = 403, description = "Forbidden - Requires a SuperAdmin session"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin Settings"
)]
pub async fn get_all_settings(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
) -> ApiResult<Vec<SettingResponse>> {
    let data = service::get_all_settings_service(&state.db).await?;
    Ok(Json(data))
}
