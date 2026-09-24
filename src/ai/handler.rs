use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use tower_sessions::Session;

use crate::ai::service::{RelayResponse, chat_service};
use crate::app_error::APIError;
use crate::app_extractor::{RequiresRole, SuperAdminRole};
use crate::app_state::AppState;
use crate::auth::session_admin_id;

#[utoipa::path(
    post,
    path = "/admin/ai/chat",
    description = "Relay a prompt to the Gemini assistant. The request body is passed through unchanged and the\nGemini response (status code and body) is returned unchanged. Only super admins may call this,\nand every admin has a per-day message limit.",
    responses(
        (status = 200, description = "Gemini response relayed unchanged"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 403, description = "Forbidden - Super admin role required"),
        (status = 429, description = "Daily message limit exceeded"),
        (status = 504, description = "Upstream Gemini request timed out")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin AI Assistant"
)]
pub async fn chat(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    body: Bytes,
) -> Result<Response, APIError> {
    let admin_id = session_admin_id(session).await?;

    let RelayResponse {
        status,
        content_type,
        body,
    } = chat_service(&state, admin_id, body).await?;

    let mut response = axum::response::Response::new(axum::body::Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert(CONTENT_TYPE, content_type);
    Ok(response)
}