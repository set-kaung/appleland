use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
    http::StatusCode,
};

use tower_sessions::Session;
use tracing;

use crate::{app_error::APIError, auth::AdminRole};

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(val)) => Ok(Self(val)),
            Err(rejection) => {
                tracing::debug!("JSON parse error: {:?}", rejection);
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Invalid request body".to_string(),
                ))
            }
        }
    }
}

pub struct SaleAdminRole;
pub struct PlaygroundAdminRole;
pub struct SuperAdminRole;

pub trait IsAdminRole {
    fn get_role() -> AdminRole;
}

impl IsAdminRole for SaleAdminRole {
    fn get_role() -> AdminRole {
        AdminRole::SaleAdmin
    }
}

impl IsAdminRole for PlaygroundAdminRole {
    fn get_role() -> AdminRole {
        AdminRole::PlaygroundAdmin
    }
}

impl IsAdminRole for SuperAdminRole {
    fn get_role() -> AdminRole {
        AdminRole::SuperAdmin
    }
}

pub struct RequiresRole<T>(pub AdminRole, pub std::marker::PhantomData<T>);

impl<S, T> FromRequestParts<S> for RequiresRole<T>
where
    S: Send + Sync,
    T: IsAdminRole,
{
    type Rejection = APIError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(APIError::InternalErr)?;

        let role: AdminRole = session
            .get("role")
            .await
            .map_err(|_| APIError::InternalErr)?
            .ok_or(APIError::Forbidden)?;

        let required = T::get_role();

        if role == required || role == AdminRole::SuperAdmin {
            Ok(RequiresRole(role, std::marker::PhantomData))
        } else {
            Err(APIError::Forbidden)
        }
    }
}
