use super::auth::{AdminRole, AuthError, Claims};
use crate::app_error::APIError;
use crate::app_error::AppError;
use crate::app_state::AppState;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
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
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::Session;
use tracing::{Span, info};

pub async fn user_auth_middleware(
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

    let auth_arc = state.auth;
    request
        .extensions_mut()
        .insert::<Arc<dyn crate::auth::AuthProvider>>(Arc::clone(&auth_arc));

    match (*auth_arc).verify(&token).await {
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
    tracing::info!("{} {} {}", response.status(), method, path);
    response
}

pub async fn partitioned_cookie_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    if let Some(set_cookie) = response.headers().get(header::SET_COOKIE)
        && let Ok(val) = set_cookie.to_str()
        && val.starts_with("id=")
        && !val.contains("Partitioned")
    {
        let patched = format!("{}; Partitioned", val);
        match HeaderValue::from_str(&patched) {
            Ok(header_val) => {
                response
                    .headers_mut()
                    .insert(header::SET_COOKIE, header_val);
            }
            Err(e) => {
                tracing::error!("failed to build Partitioned cookie header: {e}");
                return APIError::InternalErr.into_response();
            }
        }
    }

    response
}

pub fn with_global_middleware(
    router: Router,
    allowed_origins: Vec<HeaderValue>,
    burst_size: u32,
    per_sec: u64,
) -> Result<Router, AppError> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(per_sec)
        .burst_size(burst_size)
        .finish()
        .ok_or_else(|| AppError::StartupErr("failed to setup governor middleware".to_string()))?;

    let rate_limit_layer = GovernorLayer::new(governor_conf);

    let cors_layer = CorsLayer::new().allow_origin(allowed_origins);

    Ok(router.layer(
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
            .layer(
                cors_layer
                    .allow_methods([
                        Method::GET,
                        Method::POST,
                        Method::PUT,
                        Method::PATCH,
                        Method::DELETE,
                    ])
                    .allow_headers(vec![CONTENT_TYPE, AUTHORIZATION, ACCEPT])
                    .allow_credentials(true),
            )
            .layer(axum::middleware::from_fn(debug_log_middleware))
            .layer(TraceLayer::new_for_http().on_failure(
                |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    tracing::error!("Request failed: {:?}", error);
                },
            )),
    ))
}

pub async fn admin_auth_middleware(
    State(_): State<AppState>,
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Response {
    let role = session.get::<AdminRole>("role").await;
    if let Ok(v) = role {
        match v {
            Some(_) => next.run(request).await,
            None => {
                tracing::info!("no session found");
                APIError::Forbidden.into_response()
            }
        }
    } else {
        APIError::InternalErr.into_response()
    }
}
