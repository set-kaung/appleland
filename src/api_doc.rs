use crate::product::model::{
    AdminProduct, CreateProductPayload, UpdateProductPayload, UserProduct,
};

use crate::category::model::{AdminCategory, CreateCategoryPayload};
use crate::sales::model::{AdminSale, AdminSaleDetails};

use crate::auth::password::{Admin, LoginRequest};

use super::category;
use super::inventory;
use super::sales;
use super::pagination::*;
use super::product;
use crate::auth::password;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "API to be consumed by user applications"),
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
    ),
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    info(description = "API to be consumed by admin dashboar"),
    paths(
        product::handler::get_all_products,
        product::handler::create_product,
        product::handler::update_product,
        product::handler::delete_product,
        inventory::handler::update_inventory,
        password::admin_login_handler,
        password::admin_logout_handler,
        category::handler::create_category,
        sales::handler::get_sales_paginated,
        sales::handler::get_sale_detail_by_id,
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
            AdminSale,
            AdminSaleDetails,
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
