use super::auth::admin::model::AdminRole;
use super::auth::{AuthError, Claims};
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
use uuid::Uuid;

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
            tracing::warn!("token expired");
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(AuthError::InvalidToken) => {
            tracing::warn!("invalid token");
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(AuthError::Provider(e)) => {
            tracing::error!(error = %e, "auth provider error");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn user_row_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let Some(claims) = request.extensions().get::<Claims>().cloned() else {
        tracing::error!("user_row_middleware ran without verified claims");
        return APIError::InternalErr.into_response();
    };

    let supabase_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id =
        match crate::users::service::find_user_id_by_supabase_id(&state.db, supabase_id).await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!(error = %e, "failed to resolve supabase user");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let Some(user_id) = user_id else {
        tracing::info!(supabase_id = %supabase_id, "account setup required");
        return APIError::AccountSetupRequired.into_response();
    };

    request.extensions_mut().insert(crate::auth::CurrentUser {
        id: user_id,
        supabase_id,
    });
    next.run(request).await
}

#[derive(Debug, Clone, Copy)]
pub struct RequestId(pub Uuid);

pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4);

    request.extensions_mut().insert(RequestId(request_id));

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id.to_string()) {
        response.headers_mut().insert("x-request-id", value);
    }
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
            .layer(axum::middleware::from_fn(request_id_middleware))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request| {
                        let request_id = request
                            .extensions()
                            .get::<RequestId>()
                            .map_or_else(|| "unattributed".to_string(), |id| id.0.to_string());
                        tracing::debug_span!(
                            "http_request",
                            request_id = %request_id,
                            method = %request.method(),
                            path = %request.uri().path(),
                        )
                    })
                    .on_response(|response: &Response, latency: Duration, _span: &Span| {
                        tracing::debug!(
                            status = response.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "request completed"
                        );
                    })
                    .on_failure(
                        |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                            tracing::error!(
                                error = ?error,
                                latency_ms = latency.as_millis() as u64,
                                "request failed"
                            );
                        },
                    ),
            ),
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
