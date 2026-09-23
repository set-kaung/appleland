use super::model::{Admin, LoginRequest};
use super::store;
use crate::app_error::StoreError::RecordNotFound;
use crate::{app_error::APIError, app_state::AppState};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt},
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

/// Fixed 16-byte zero salt encoded as base64, used only to burn time on unknown usernames
/// so that login attempts take roughly the same duration regardless of whether the user exists.
const DUMMY_SALT_B64: &str = "AAAAAAAAAAAAAAAAAAAAAA";

#[utoipa::path(
    post,
    path = "/admin/login",
    description = "Authenticate an admin and establish a session cookie.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authenticated admin details", body = Admin),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
#[axum::debug_handler]
pub async fn admin_login_handler(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<Admin>), APIError> {
    match store::get_admin_by_username(&state.db, payload.username).await {
        Ok(admin) => {
            let admin_resp: Admin = admin.clone().into();
            let parsed_hash = PasswordHash::new(&admin.password_hash).map_err(|e| {
                tracing::error!("faied to parse password hash: {}", e);
                APIError::InternalErr
            })?;

            let is_valid = Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .is_ok();

            if !is_valid {
                Err(APIError::AdminInvalidCreds)
            } else {
                session.insert("id", admin.id).await.map_err(|e| {
                    tracing::error!("failed to insert ID into session: {}", e);
                    APIError::InternalErr
                })?;

                session.insert("role", admin.role).await.map_err(|e| {
                    tracing::error!("failed to insert role into session: {}", e);
                    APIError::InternalErr
                })?;

                // Rotate the session ID after privilege escalation to prevent session fixation.
                session.cycle_id().await.map_err(|e| {
                    tracing::error!("failed to rotate session id: {}", e);
                    APIError::InternalErr
                })?;

                Ok((StatusCode::OK, Json(admin_resp)))
            }
        }
        Err(e) => {
            tracing::debug!("err admin: {}", e);
            match e {
                RecordNotFound => {
                    // Avoid timing side channel attacks
                    let dummy_salt = Salt::from_b64(DUMMY_SALT_B64).map_err(|e| {
                        tracing::error!("failed to build dummy salt: {}", e);
                        APIError::InternalErr
                    })?;
                    let _ = Argon2::default()
                        .hash_password(b"dummy", dummy_salt)
                        .map_err(|e| {
                            tracing::error!("failed to hash dummy password: {}", e);
                            APIError::InternalErr
                        })?;
                    Err(APIError::AdminInvalidCreds)
                }
                _ => Err(APIError::InternalErr),
            }
        }
    }
}
#[utoipa::path(
    post,
    path = "/admin/logout",
    description = "Destroy the current admin session.",
    responses(
        (status = 200, description = "Session destroyed"),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Authentication"
)]
pub async fn admin_logout_handler(session: Session) -> Result<StatusCode, APIError> {
    session
        .delete()
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete session: {}", e);
            APIError::InternalErr
        })
        .map(|_| StatusCode::OK)
}

pub async fn session_admin_id(session: Session) -> Result<Uuid, APIError> {
    session
        .get::<Uuid>("id")
        .await
        .map_err(|e| {
            error!("session error: {e}");
            APIError::InternalErr
        })?
        .ok_or(APIError::Unauthorized)
}
