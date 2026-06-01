use super::app_state::AppState;
use super::auth::{AuthError, Claims};
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::HeaderValue;
use axum::{
    body::Body,
    extract::Request,
    extract::State,
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use std::sync::Arc;
use std::time::Duration;
use tower::timeout::TimeoutLayer;
use tower::{BoxError, ServiceBuilder};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::info;

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
            if claims.role != "authenticated" {
                info!("unathenticated access with role: {}", claims.role);
                return StatusCode::FORBIDDEN.into_response();
            }
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

pub async fn debug_log_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    tracing::debug!("{} {} {}", response.status(), method, path);
    response
}

pub fn with_global_middleware(
    router: Router,
    allowed_origins: Vec<HeaderValue>,
    burst_size: u32,
    per_sec: u64,
) -> Router {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(per_sec)
        .burst_size(burst_size)
        .finish()
        .unwrap();
    let rate_limit_layer = GovernorLayer::new(governor_conf);

    let cors_layer = if allowed_origins
        .iter()
        .any(|h| h == &HeaderValue::from_static("*"))
    {
        CorsLayer::new().allow_origin(AllowOrigin::any())
    } else {
        CorsLayer::new().allow_origin(allowed_origins)
    };

    router.layer(
        ServiceBuilder::new()
            .layer(rate_limit_layer)
            .layer(HandleErrorLayer::new(|err: BoxError| async move {
                if err.is::<tower::timeout::error::Elapsed>() {
                    (StatusCode::GATEWAY_TIMEOUT, "request timed out".to_string())
                } else {
                    tracing::error!("internal server error: {err}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(cors_layer.allow_methods(Any).allow_headers(Any))
            .layer(axum::middleware::from_fn(debug_log_middleware)),
    )
}
