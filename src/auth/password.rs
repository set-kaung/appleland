use crate::app_error::APIError;
use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::query_as;
use std::str::FromStr;
use time::OffsetDateTime;
use tower_sessions::Session;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use uuid::Uuid;

use crate::{
    app_error::{self, StoreError::NoRows},
    app_state::AdminState,
};

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

impl ToString for AdminRole {
    fn to_string(&self) -> String {
        match self {
            AdminRole::SaleAdmin => "sale_admin".to_string(),
            AdminRole::PlaygroundAdmin => "playground_admin".to_string(),
            AdminRole::SuperAdmin => "super_admin".to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(sqlx::FromRow, Clone)]
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
        sqlx::Error::RowNotFound => app_error::StoreError::NoRows,
        _ => {
            tracing::debug!("{}", e);
            app_error::StoreError::Database(e)
        }
    })
}

#[axum::debug_handler]
pub async fn admin_login_handler(
    State(state): State<AdminState>,
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
                NoRows => Err(APIError::AdminInvalidCreds),
                _ => Err(APIError::InternalErr),
            }
        }
    }
}

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

/// log_err will log the error `e` and convert to `err_variant`
#[macro_export]
macro_rules! log_err {
    ($e:expr, $err_variant:expr) => {
        $e.map_err(|e| {
            tracing::error!("error: {}", e);
            $err_variant
        })
    };
}
