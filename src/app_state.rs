use axum::extract::FromRef;
use sqlx::PgPool;

use crate::auth::AuthProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub api_state: ApiState,
    pub admin_state: AdminState,
}

#[derive(Clone)]
pub struct ApiState {
    auth: Arc<dyn AuthProvider>,
    pub db: PgPool,
}

impl ApiState {
    pub fn new(auth: Arc<dyn AuthProvider>, db: PgPool) -> Self {
        ApiState { auth, db }
    }

    pub fn auth_arc(&self) -> Arc<dyn AuthProvider> {
        Arc::clone(&self.auth)
    }
}

impl FromRef<AppState> for ApiState {
    fn from_ref(app: &AppState) -> Self {
        app.api_state.clone()
    }
}

#[derive(Clone)]
pub struct AdminState {
    pub db: PgPool,
}

impl AdminState {
    pub fn new(db: PgPool) -> Self {
        AdminState { db }
    }
}

impl FromRef<AppState> for AdminState {
    fn from_ref(app: &AppState) -> Self {
        app.admin_state.clone()
    }
}

pub trait AccessDB {
    fn access_db(&self) -> &PgPool;
}

impl AccessDB for ApiState {
    fn access_db(&self) -> &PgPool {
        &self.db
    }
}

impl AccessDB for AdminState {
    fn access_db(&self) -> &PgPool {
        &self.db
    }
}
