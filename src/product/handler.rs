use log::error;
use uuid::Uuid;

use super::{
    model::{CreateProductPayload, Product},
    store,
};
use crate::{app_state::AppState, product::model::UpdateProductPayload};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::core::pagination::*;

pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<Product>>, (StatusCode, String)> {
    let (limit, offset, current_page) = query.get_limit_and_offset();

    store::fetch_paged(&state.db, limit, offset)
        .await
        .map(|(data, total_items)| {
            let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
            Json(PaginatedResponse {
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
    State(state): State<AppState>,
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
    State(state): State<AppState>,
    Json(payload): Json<CreateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    store::create(&state.db, payload)
        .await
        .map(|product| (StatusCode::CREATED, Json(product)))
        .map_err(|err| {
            error!("Failed to create product: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create product".to_string(),
            )
        })
}

pub async fn update_product(
    State(state): State<AppState>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    store::update(&state.db, payload)
        .await
        .map(|product| (StatusCode::OK, Json(product)))
        .map_err(|err| {
            error!("Failed to update product: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update product".to_string(),
            )
        })
}
