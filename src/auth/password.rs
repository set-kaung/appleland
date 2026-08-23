use crate::{app_error::APIError, app_state::AppState};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::str::FromStr;
use time::OffsetDateTime;
use tower_sessions::Session;
use utoipa::ToSchema;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt},
};

/// Fixed 16-byte zero salt encoded as base64, used only to burn time on unknown usernames
/// so that login attempts take roughly the same duration regardless of whether the user exists.
const DUMMY_SALT_B64: &str = "AAAAAAAAAAAAAAAAAAAAAA==";
use tracing::error;
use uuid::Uuid;

use crate::app_error::{self, StoreError::RecordNotFound};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    ToSchema,
)]
#[sqlx(type_name = "admin_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    SaleAdmin,
    PlaygroundAdmin,
    SuperAdmin,
}

impl FromStr for AdminRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sale_admin" => Ok(AdminRole::SaleAdmin),
            "playground_admin" => Ok(AdminRole::PlaygroundAdmin),
            "super_admin" => Ok(AdminRole::SuperAdmin),
            _ => Err("not defined role".to_string()),
        }
    }
}

// impl ToString for AdminRole {
//     fn to_string(&self) -> String {
//         match self {
//             AdminRole::SaleAdmin => "sale_admin".to_string(),
//             AdminRole::PlaygroundAdmin => "playground_admin".to_string(),
//             AdminRole::SuperAdmin => "super_admin".to_string(),
//         }
//     }
// }

impl std::fmt::Display for AdminRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self {
            AdminRole::SaleAdmin => "sale_admin",
            AdminRole::PlaygroundAdmin => "playground_admin",
            AdminRole::SuperAdmin => "super_admin",
        };
        write!(f, "{}", str_rep)
    }
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(sqlx::FromRow, Clone)]
#[allow(dead_code)]
struct AdminRow {
    id: Uuid,
    username: String,
    password_hash: String,
    role: AdminRole,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Admin {
    pub id: Uuid,
    pub username: String,
    pub role: AdminRole,
}

impl From<AdminRow> for Admin {
    fn from(value: AdminRow) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role,
        }
    }
}

async fn get_admin_by_username(
    pool: &sqlx::PgPool,
    username: String,
) -> Result<AdminRow, app_error::StoreError> {
    query_as!(
        AdminRow,
        "
    SELECT id,username,password_hash,role as \"role: AdminRole\",created_at,updated_at
    FROM admins
    WHERE username = $1",
        username
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => app_error::StoreError::RecordNotFound,
        _ => {
            tracing::debug!("{}", e);
            app_error::StoreError::Database(e)
        }
    })
}

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
    match get_admin_by_username(&state.db, payload.username).await {
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
                    let _ = Argon2::default().hash_password(b"dummy", dummy_salt).map_err(|e| {
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
