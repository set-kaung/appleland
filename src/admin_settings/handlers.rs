use crate::admin_settings::model::{SettingResponse, UpdateSettingPayload};
use crate::admin_settings::service;
use crate::app_extractor::{RequiresRole, SuperAdminRole, ValidatedJson};
use crate::app_state::AppState;
use crate::auth::session_admin_id;
use crate::ApiResult;
use axum::Json;
use axum::extract::{Path, State};
use tower_sessions::Session;

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

#[utoipa::path(
    patch,
    path = "/admin/settings/{key}",
    description = "Update the value of an existing setting. The value must match the setting's declared type (string, integer, decimal, or boolean); otherwise the request is rejected with 400. Only the value is changed; the setting key, description, and type are untouched.",
    params(
        ("key" = String, Path, description = "The setting key to update")
    ),
    request_body = UpdateSettingPayload,
    responses(
        (status = 200, description = "The updated setting", body = SettingResponse),
        (status = 400, description = "Value does not match the setting's declared type"),
        (status = 403, description = "Forbidden - Requires a SuperAdmin session"),
        (status = 404, description = "Setting not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("session_cookie" = [])),
    tag = "Admin Settings"
)]
pub async fn update_setting_value(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    Path(key): Path<String>,
    session: Session,
    ValidatedJson(payload): ValidatedJson<UpdateSettingPayload>,
) -> ApiResult<SettingResponse> {
    let admin_id = session_admin_id(session).await?;
    let data =
        service::update_setting_value_service(&state.db, &key, payload.value, admin_id).await?;
    Ok(Json(data))
}
