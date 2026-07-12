use super::store;
use crate::app_error::APIError;
use crate::app_extractor::{self, RequiresRole, SaleAdminRole};
use crate::inventory::store as inventory_store;
use crate::pagination;
use axum::extract::Path;
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

use crate::app_state::{self, AppState};
use crate::product::model::{
    AdminProduct, CreateProductPayload, InsertProduct, UpdateProduct, UpdateProductPayload,
    UserProduct,
};

use crate::auth::password::session_admin_id;
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
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<UserProduct>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
pub async fn get_products_paginated(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<UserProduct>>, APIError> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paged(&state.db, limit, offset, true)
        .await
        .map(|(data, total_items)| {
            let data = data.into_iter().map(Into::into).collect();
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
            APIError::InternalErr
        })
}

#[utoipa::path(
    get,
    path = "/admin/products",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<UserProduct>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
pub async fn get_products_paginated_admin(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> Result<Json<pagination::PaginatedResponse<AdminProduct>>, APIError> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paged_admin(&state.db, limit, offset)
        .await
        .map(|(data, total_items)| {
            let data = data.into_iter().map(Into::into).collect::<Vec<_>>();
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
            APIError::InternalErr
        })
}

#[utoipa::path(
    get,
    path = "/admin/products/all",
    responses(
        (status = 200, body = Vec<AdminProduct>),
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
) -> Result<Json<Vec<AdminProduct>>, APIError> {
    store::fetch_all(&state.db)
        .await
        .map(|result| Json(result.into_iter().map(Into::into).collect()))
        .map_err(|err| {
            error!("failed to get all products: {err}");
            APIError::InternalErr
        })
}

#[utoipa::path(
    get,
    path = "/api/products/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 200, body = UserProduct),
        (status = 404, description = "Product not found"),
        (status = 500)
    ),
    tag = "Products"
)]
pub async fn get_product_by_id(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<UserProduct>, APIError> {
    let product: UserProduct = store::fetch_by_id(&state.db, product_id).await?.into();
    Ok(Json(product))
}

#[utoipa::path(
    post,
    path = "/admin/products",
    request_body = CreateProductPayload,
    responses(
        (status = 201, body = AdminProduct),
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
    State(state): State<app_state::AppState>,
    session: Session,
    app_extractor::ValidatedJson(payload): app_extractor::ValidatedJson<CreateProductPayload>,
) -> Result<(StatusCode, Json<AdminProduct>), APIError> {
    let admin_id: Uuid = session_admin_id(session).await?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        APIError::InternalErr
    })?;

    let insert_payload = InsertProduct {
        name: payload.name,
        barcode: payload.barcode,
        description: payload.description,
        category_id: payload.category_id,
        sub_category_id: payload.sub_category_id,
        is_active: payload.is_active,
        price: payload.price,
        created_by: admin_id,
        updated_by: admin_id,
    };

    let product_id = store::create(&mut tx, insert_payload).await?;

    inventory_store::insert_new_inventory_record(product_id, admin_id, 0, &mut tx).await?;

    tx.commit().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        APIError::InternalErr
    })?;

    let product: AdminProduct = store::fetch_by_id(&state.db, product_id).await?.into();

    Ok((StatusCode::CREATED, Json(product)))
}
#[utoipa::path(
    put,
    path = "/admin/products/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = UpdateProductPayload,
    responses(
        (status = 200, body = AdminProduct),
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
) -> Result<(StatusCode, Json<AdminProduct>), APIError> {
    let admin_id: Uuid = session_admin_id(session).await?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        APIError::InternalErr
    })?;

    let update_payload = UpdateProduct {
        id: product_id,
        barcode: payload.barcode,
        name: payload.name,
        description: payload.description,
        category_id: payload.category_id,
        sub_category_id: payload.sub_category_id,
        is_active: payload.is_active,
        price: payload.price,
        updated_by: admin_id,
    };

    let product = store::update(&mut tx, update_payload).await?.into();
    tx.commit().await.map_err(|e| {
        error!("failed to commit transaction: {e}");
        APIError::InternalErr
    })?;
    Ok((StatusCode::OK, Json(product)))
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
) -> Result<StatusCode, APIError> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        APIError::InternalErr
    })?;

    store::delete(&mut tx, product_id).await?;
    tx.commit().await.map_err(|e| {
        error!("failed to commit transaction: {e}");
        APIError::InternalErr
    })?;
    Ok(StatusCode::NO_CONTENT)
}
