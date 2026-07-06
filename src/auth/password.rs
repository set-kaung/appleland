use crate::{app_error::APIError, app_state::AppState};
use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::query_as;
use std::{fmt::Display, str::FromStr};
use time::OffsetDateTime;
use tower_sessions::Session;
use utoipa::ToSchema;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use tracing::error;
use uuid::Uuid;

use crate::app_error::{self, StoreError::RecordNotFound};

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, sqlx::Type, serde::Serialize, serde::Deserialize,
)]
#[sqlx(type_name = "admin_role", rename_all = "snake_case")]
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

impl Display for AdminRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self {
            AdminRole::SaleAdmin => "sale_admin".to_string(),
            AdminRole::PlaygroundAdmin => "playground_admin".to_string(),
            AdminRole::SuperAdmin => "super_admin".to_string(),
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
struct Admin {
    id: Uuid,
    username: String,
    password_hash: String,
    role: AdminRole,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

async fn get_admin_by_username(
    pool: &sqlx::PgPool,
    username: String,
) -> Result<Admin, app_error::StoreError> {
    query_as!(
        Admin,
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
    request_body = LoginRequest,
    responses(
        (status = 200),
        (status = 401)
    ),
    tag = "Authentication"
)]
#[axum::debug_handler]
pub async fn admin_login_handler(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> Result<StatusCode, APIError> {
    match get_admin_by_username(&state.db, payload.username).await {
        Ok(admin) => {
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

                Ok(StatusCode::OK)
            }
        }
        Err(e) => {
            tracing::debug!("err admin: {}", e);
            match e {
                RecordNotFound => Err(APIError::AdminInvalidCreds),
                _ => Err(APIError::InternalErr),
            }
        }
    }
}
#[utoipa::path(
    post,
    path = "/admin/logout",
    responses(
        (status = 200)
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
