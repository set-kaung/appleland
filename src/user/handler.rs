use crate::app_error::APIError;
use crate::app_extractor::ValidatedJson;
use crate::app_state::AppState;
use crate::auth::Claims;
use crate::user::model::CreateUserPayload;
use crate::user::service;
use axum::Extension;
use axum::extract::State;
use uuid::Uuid;

pub async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateUserPayload>,
) -> Result<(), APIError> {
    let supabase_id = Uuid::parse_str(&claims.sub).map_err(|_| APIError::Unauthorized)?;
    service::new_user_account_creation(&state.db, supabase_id, payload).await?;
    Ok(())
}
