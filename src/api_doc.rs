use crate::product::model::{CreateProductPayload, Product, UpdateProductPayload};

use super::pagination::*;
use super::product;
use crate::auth::password;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        product::handler::get_products_paginated,
        product::handler::get_product_by_id,
        product::handler::get_all_products,
        product::handler::create_product,
        product::handler::update_product,
        product::handler::delete_product,
        password::admin_login_handler,
        password::admin_logout_handler,
    ),
    components(
        schemas(
            Product,
            CreateProductPayload,
            UpdateProductPayload,
            PaginationQuery,
            PaginatedResponse<Product>,
        )
    ),
    tags(
        (name = "Products"),
        (name = "Admin Products"),
        (name = "Authentication")
    )
)]
pub struct ApiDoc;
