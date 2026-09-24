use super::store;
use crate::app_error::APIError;
use crate::app_extractor::{RequiresRole, SaleAdminRole, SuperAdminRole, ValidatedJson};
use crate::inventory::store as inventory_store;
use crate::{ApiResult, PaginatedApiResult, pagination};
use axum::extract::Path;
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::product::model::{
    AdminProduct, CreateProductPayload, InsertProduct, ProductFilter, ProductSearchParamsAdmin,
    UpdateProduct, UpdateProductPayload, UserProduct,
};

use crate::auth::session_admin_id;
use crate::inventory::model::NewInventoryRecord;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

#[utoipa::path(
    get,
    path = "/api/products",
    description = "List all products shown in the online store with pagination.",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<UserProduct>),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Products"
)]
pub async fn get_products_paginated(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> PaginatedApiResult<UserProduct> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paginated(&state.db, limit, offset, true)
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
    description = "List all products (shown online or not) with pagination.",
    params(pagination::PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of products", body = pagination::PaginatedResponse<AdminProduct>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn get_products_paginated_admin(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
) -> PaginatedApiResult<AdminProduct> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paginated_admin(&state.db, limit, offset)
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
    description = "Fetch every product record without pagination.",
    responses(
        (status = 200, description = "List of all products", body = Vec<AdminProduct>),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn get_all_products(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
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
    description = "Get a single product by its UUID.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 200, description = "Product details", body = UserProduct),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Products"
)]
pub async fn get_product_by_id(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> ApiResult<UserProduct> {
    let product: UserProduct = store::fetch_by_id(&state.db, product_id).await?.into();
    Ok(Json(product))
}

#[utoipa::path(
    get,
    path = "/admin/products/{product_id}",
    description = "Get a single product (admin view) by its UUID.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 200, description = "Product details", body = AdminProduct),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn get_product_by_id_admin(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> ApiResult<AdminProduct> {
    let product: AdminProduct = store::fetch_by_id(&state.db, product_id).await?.into();
    Ok(Json(product))
}

#[utoipa::path(
    post,
    path = "/admin/products",
    description = "Create a new product and optionally seed its initial inventory.",
    request_body = CreateProductPayload,
    responses(
        (status = 201, description = "Product created", body = AdminProduct),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 409, description = "Conflict - Product already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn create_product(
    State(state): State<AppState>,
    session: Session,
    ValidatedJson(payload): ValidatedJson<CreateProductPayload>,
) -> Result<(StatusCode, Json<AdminProduct>), APIError> {
    payload.validate()?;
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
        is_shown_online: payload.is_shown_online,
        selling_price: payload.selling_price,
        image_url: payload.image_url,
        created_by: admin_id,
        updated_by: admin_id,
        is_perishable: payload.is_perishable,
    };

    let product_id = store::create(&mut tx, insert_payload).await?;
    if let Some(inventory_payload) = payload.inventory {
        let inventory_record = NewInventoryRecord::new(
            product_id,
            admin_id,
            payload.is_perishable,
            &inventory_payload,
        )?;
        inventory_store::insert_new_inventory_record(&mut *tx, inventory_record).await?;
    }

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
    description = "Update an existing product.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    request_body = UpdateProductPayload,
    responses(
        (status = 200, description = "Product updated", body = AdminProduct),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - Missing or invalid session"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn update_product(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<(StatusCode, Json<AdminProduct>), APIError> {
    payload.validate()?;
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
        is_shown_online: payload.is_shown_online,
        selling_price: payload.selling_price,
        image_url: payload.image_url,
        updated_by: admin_id,
        is_perishable: payload.is_perishable,
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
    description = "Soft-delete a product by its UUID.",
    params(
        ("product_id" = Uuid, Path, description = "Product UUID")
    ),
    responses(
        (status = 204, description = "Product soft-deleted"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]

pub async fn delete_product(
    RequiresRole(_role, _): RequiresRole<SuperAdminRole>,
    State(state): State<AppState>,
    session: Session,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, APIError> {
    let admin_id = session_admin_id(session).await?;
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("failed to create transaction: {e}");
        APIError::InternalErr
    })?;

    store::delete(&mut tx, product_id, admin_id).await?;
    tx.commit().await.map_err(|e| {
        error!("failed to commit transaction: {e}");
        APIError::InternalErr
    })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/admin/products/search/advanced",
    description = "Advanced product search for admins with full filter support.",
    request_body = ProductSearchParamsAdmin,
    params(pagination::PaginationQuery),
    responses(
        (
            status = 200,
            description = "Paginated list of matching products",
            body = pagination::PaginatedResponse<AdminProduct>
        ),
        (status = 400, description = "Invalid query parameters"),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal database error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn search_products_admin_advanced(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
    ValidatedJson(search_params): ValidatedJson<ProductSearchParamsAdmin>,
) -> PaginatedApiResult<AdminProduct> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (values, total_items) =
        store::fetch_products_filtered(&state.db, &search_params, limit, offset).await?;
    let data = values.into_iter().map(|x| x.into()).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}

#[utoipa::path(
    get,
    path = "/admin/products/search",
    description = "Simple admin product search by query string and optional category filter.",
    params(
        pagination::PaginationQuery,
        ProductFilter
    ),
    responses(
        (
            status = 200,
            description = "Paginated list of matching products",
            body = pagination::PaginatedResponse<AdminProduct>
        ),
        (status = 400, description = "Invalid query parameters"),
        (status = 403, description = "Forbidden - Missing or invalid session"),
        (status = 500, description = "Internal database error")
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "Admin Products"
)]
pub async fn search_products_admin(
    RequiresRole(_role, _): RequiresRole<SaleAdminRole>,
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
    Query(product_query): Query<ProductFilter>,
) -> PaginatedApiResult<AdminProduct> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (values, total_items) =
        store::fetch_products_search(&state.db, product_query, limit, offset).await?;
    let data = values.into_iter().map(|x| x.into()).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}

#[utoipa::path(
    get,
    path = "/api/products/search",
    description = "Search the public product catalog by name, barcode, category, sub-category, and price range.",
    params(
        pagination::PaginationQuery,
        ProductFilter
    ),
    responses(
        (
            status = 200,
            description = "Paginated list of matching products",
            body = pagination::PaginatedResponse<UserProduct>
        ),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT"),
        (status = 403, description = "Forbidden - Insufficient privileges"),
        (status = 500, description = "Internal database error")
    ),
    security(
        ("bearer" = [])
    ),
    tag = "Products"
)]
pub async fn search_products(
    State(state): State<AppState>,
    Query(query): Query<pagination::PaginationQuery>,
    Query(product_query): Query<ProductFilter>,
) -> PaginatedApiResult<UserProduct> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    let (values, total_items) =
        store::fetch_products_search(&state.db, product_query, limit, offset).await?;
    let data = values.into_iter().map(|x| x.into()).collect();
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    Ok(Json(pagination::PaginatedResponse {
        data,
        total_items,
        total_pages,
        current_page,
    }))
}
