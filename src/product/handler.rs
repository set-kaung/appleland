use super::store;
use crate::app_error::StoreError;
use crate::app_extractor;
use uuid::Uuid;

use crate::pagination;
use tracing::{debug, error};

use super::model::{CreateProductPayload, Product};
use crate::app_state;
use crate::product::model::UpdateProductPayload;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

pub async fn list_products(
    State(state): State<app_state::AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<Product>>, (StatusCode, String)> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paged(&state.db, limit, offset)
        .await
        .map(|(data, total_items)| {
            let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
            Json(pagination::PaginatedResponse {
                data,
                total_items,
                total_pages,
                current_page,
            })
        })
        .map_err(|err| {
            error!("failed to fetch products in list_products: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch products".to_string(),
            )
        })
}

pub async fn get_product_by_id(
    State(state): State<app_state::AppState>,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Product>, (StatusCode, String)> {
    store::fetch_by_id(&state.db, product_id)
        .await
        .map(Json)
        .map_err(|err| {
            error!("failed to fetch product by id: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch product by id".to_string(),
            )
        })
}

pub async fn create_product(
    State(state): State<app_state::AppState>,
    app_extractor::ValidatedJson(payload): app_extractor::ValidatedJson<CreateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    match store::create(&mut tx, payload).await {
        Ok(product) => {
            tx.commit().await.map_err(|e| {
                error!("failed to commit to database in create_product: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_string(),
                )
            })?;

            Ok((StatusCode::CREATED, Json(product)))
        }
        Err(sqlx::Error::Database(db_err)) => {
            if let Some(pg_err) = db_err
                .as_error()
                .downcast_ref::<sqlx::postgres::PgDatabaseError>()
            {
                match pg_err.code() {
                    "23505" => {
                        debug!("error: {:?}", pg_err.column().unwrap_or("unknown"));
                        return Err((
                            StatusCode::CONFLICT,
                            "a product with the given key already exists".to_string(),
                        ));
                    }
                    "23502" => {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            format!(
                                "Required field is missing: {}",
                                pg_err.column().unwrap_or("unknown")
                            ),
                        ));
                    }
                    _ => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error occurred".into(),
                    )),
                }
            } else {
                error!("failed to create product: {db_err}");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".into(),
                ))
            }
        }
        Err(err) => {
            error!("Failed to create product: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create product".to_string(),
            ))
        }
    }
}

pub async fn update_product(
    State(state): State<app_state::AppState>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "something went wrong".to_string(),
        )
    })?;

    match store::update(&mut tx, payload).await {
        Ok(result) => {
            tx.commit().await.map_err(|e| {
                error!("failed to commit to database in update_product: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_string(),
                )
            })?;

            Ok((StatusCode::OK, Json(result)))
        }
        Err(err) => {
            error!("failed to update product: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update product".to_string(),
            ))
        }
    }
}

pub async fn delete_product(
    State(state): State<app_state::AppState>,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "something went wrong".to_string(),
        )
    })?;

    match store::delete(&mut tx, product_id).await {
        Ok(true) => {
            tx.commit().await.map_err(|e| {
                error!("failed to commit transaction in delete_product: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_string(),
                )
            })?;

            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Ok(StatusCode::NOT_FOUND),
        Err(StoreError::SafetyTrigger(count)) => {
            error!("tried to delete more than one instance: {count}");
            Err((StatusCode::CONFLICT, "Failed to delete product".to_string()))
        }
        Err(StoreError::Database(err)) => {
            error!("failed delete product; database error: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete product".to_string(),
            ))
        }
    }
}
