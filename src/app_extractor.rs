use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
};

use tracing;

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
