use crate::product::model::{
    AdminProduct, CreateProductPayload, UpdateProductPayload, UserProduct,
};

use crate::category::model::{
    CreateCategoryPayload,
    AdminCategory
};

use crate::auth::password::{Admin,LoginRequest};

use super::inventory;
use super::pagination::*;
use super::product;
use super::category;
use crate::auth::password;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        product::handler::get_products_paginated,
        product::handler::get_product_by_id,
    ),
    components(
        schemas(
            UserProduct,
            PaginationQuery,
            PaginatedResponse<UserProduct>,
        )
    ),
    tags(
        (name = "Products")
    )
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        product::handler::get_all_products,
        product::handler::create_product,
        product::handler::update_product,
        product::handler::delete_product,
        inventory::handler::update_inventory,
        password::admin_login_handler,
        password::admin_logout_handler,
        category::handler::create_category,
    ),
    components(
        schemas(
            AdminProduct,
            CreateProductPayload,
            UpdateProductPayload,
            PaginationQuery,
            PaginatedResponse<UserProduct>,
            CreateCategoryPayload,
            AdminCategory,
            Admin,
            LoginRequest,
        )
    ),
    tags(
        (name = "Admin Products"),
        (name = "Authentication")
    )
)]
pub struct AdminApiDoc;
