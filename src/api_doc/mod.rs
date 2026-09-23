pub mod admin;
pub mod user;

pub use admin::AdminApiDoc;
pub use user::ApiDoc;

use std::path::Path;
use utoipa::openapi::Components;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::app_error::AppError;

pub struct BearerSecurityAddon;

impl Modify for BearerSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.security_schemes.insert(
            "bearer".to_string(),
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

pub struct SessionCookieSecurityAddon;

impl Modify for SessionCookieSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.security_schemes.insert(
            "session_cookie".to_string(),
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
        );
    }
}

pub fn generate_docs() -> Result<(), AppError> {
    let openapi = ApiDoc::openapi();
    let admin_api = AdminApiDoc::openapi();

    let yaml = openapi
        .to_yaml()
        .map_err(|e| AppError::DocumentGenerationError(e.to_string()))?;
    let admin_yaml = admin_api
        .to_yaml()
        .map_err(|e| AppError::DocumentGenerationError(e.to_string()))?;

    let base_path = Path::new("docs/openapi");
    let admin_dir = base_path.join("admin");
    let user_dir = base_path.join("user");

    if let Err(err) = std::fs::create_dir_all(&admin_dir) {
        tracing::error!("failed to create admin docs directory: {err}");
    }

    if let Err(err) = std::fs::create_dir_all(&user_dir) {
        tracing::error!("failed to create user docs directory: {err}");
    }

    match std::fs::write(user_dir.join("openapi.yaml"), yaml) {
        Ok(_) => tracing::info!("user api docs generated to {}", user_dir.display()),
        Err(err) => tracing::error!("error generating api docs: {err}"),
    }

    match std::fs::write(admin_dir.join("openapi.yaml"), admin_yaml) {
        Ok(_) => tracing::info!("admin api docs generated to {}", admin_dir.display()),
        Err(err) => tracing::error!("error generating admin api docs: {err}"),
    }
    Ok(())
}
