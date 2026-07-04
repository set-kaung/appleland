use super::store;
use crate::app_error::StoreError;
use crate::app_extractor::{self, RequiresRole, SaleAdminRole};
use crate::inventory::store as inventory_store;
use crate::pagination;
use axum::extract::Path;
use tower_sessions::Session;
use tracing::{debug, error};
use uuid::Uuid;

use super::model::{CreateProductPayload, Product};
use crate::app_state::{self, AppState};
use crate::product::model::{
    InsertProduct, ProductWithInventory, UpdateProduct, UpdateProductPayload,
};

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

#[utoipa::path(
    get,
    path = "/api/products",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<ProductWithInventory>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
pub async fn get_products_paginated(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<ProductWithInventory>>, (StatusCode, String)> {
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

#[utoipa::path(
    get,
    path = "/admin/products",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<ProductWithInventory>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
pub async fn get_products_paginated_admin(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<ProductWithInventory>>, (StatusCode, String)> {
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

#[utoipa::path(
    get,
    path = "/admin/products/all",
    responses(
        (status = 200, body = Vec<ProductWithInventory>),
        (status = 401),
        (status = 403)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn get_all_products(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<app_state::AppState>,
) -> Result<Json<Vec<ProductWithInventory>>, (StatusCode, String)> {
    store::fetch_all(&state.db).await.map(Json).map_err(|err| {
        error!("failed to get all products: {err}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch all products".to_string(),
        )
    })
}

#[utoipa::path(
    get,
    path = "/api/products/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 200, body = ProductWithInventory),
        (status = 404, description = "Product not found"),
        (status = 500)
    ),
    tag = "Products"
)]
pub async fn get_product_by_id(
    State(state): State<AppState>,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<ProductWithInventory>, (StatusCode, String)> {
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

#[utoipa::path(
    post,
    path = "/admin/products",
    request_body = CreateProductPayload,
    responses(
        (status = 201, body = Product),
        (status = 400),
        (status = 401),
        (status = 409),
        (status = 500)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn create_product(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<app_state::AppState>,
    session: Session,
    app_extractor::ValidatedJson(payload): app_extractor::ValidatedJson<CreateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    let admin_id: Uuid = session
        .get::<Uuid>("id")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "no user_id in session".into()))?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let insert_payload = InsertProduct {
        name: payload.name,
        barcode: payload.barcode,
        description: payload.description,
        category: payload.category,
        sub_category: payload.sub_category,
        is_active: payload.is_active,
        price: payload.price,
        created_by: admin_id,
        updated_by: admin_id,
    };

    match store::create(&mut tx, insert_payload).await {
        Ok(product) => {
            if let Err(inv_err) =
                inventory_store::insert_new_inventory_record(product.id, admin_id, 0, &mut tx).await
            {
                error!("failed to create inventory row: {inv_err:?}");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to initialize product inventory".to_string(),
                ));
            }
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
                        Err((
                            StatusCode::CONFLICT,
                            "a product with the given key already exists".to_string(),
                        ))
                    }
                    "23502" => Err((
                        StatusCode::BAD_REQUEST,
                        format!(
                            "Required field is missing: {}",
                            pg_err.column().unwrap_or("unknown")
                        ),
                    )),
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
#[utoipa::path(
    put,
    path = "/admin/products/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = UpdateProductPayload,
    responses(
        (status = 200, body = Product),
        (status = 400),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn update_product(
    State(state): State<app_state::AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    let admin_id: Uuid = session
        .get::<Uuid>("id")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "no user_id in session".into()))?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "something went wrong".to_string(),
        )
    })?;

    let update_payload = UpdateProduct {
        id: product_id,
        barcode: payload.barcode,
        name: payload.name,
        description: payload.description,
        category: payload.category,
        sub_category: payload.sub_category,
        is_active: payload.is_active,
        price: payload.price,
        updated_by: admin_id,
    };

    match store::update(&mut tx, update_payload).await {
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

#[utoipa::path(
    delete,
    path = "/admin/products/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 204),
        (status = 404),
        (status = 409),
        (status = 500)
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]

pub async fn delete_product(
    State(state): State<AppState>,
    _session: Session,
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
        Err(StoreError::NoRows) => Ok(StatusCode::NOT_FOUND),
        Err(StoreError::NoRowsAffected) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "something went wrong".to_string(),
        )),
    }
}
