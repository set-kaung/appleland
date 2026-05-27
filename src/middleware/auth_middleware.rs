use crate::app_state::AppState;
use crate::auth::{AuthError, Claims};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let Some(token) = token else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let auth_arc = state.auth_arc();
    request
        .extensions_mut()
        .insert::<Arc<dyn crate::auth::AuthProvider>>(Arc::clone(&auth_arc));

    match (&*auth_arc).verify(&token).await {
        Ok(claims) => {
            request.extensions_mut().insert::<Claims>(claims);
            next.run(request).await
        }
        Err(AuthError::Expired) => {
            println!("token expired");
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(AuthError::InvalidToken) => {
            println!("invalid token");
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(AuthError::Provider(e)) => {
            println!("internal error : {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
