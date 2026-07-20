use crate::product::model::{
    AdminProduct, CreateProductPayload, UpdateProductPayload, UserProduct,
};

use crate::category::model::{AdminCategory, CreateCategoryPayload, UpdateCategoryPayload};
use crate::inventory::model::{AdminInventory, NewInventoryPayload};
use crate::sales::model::{AdminSale, AdminSaleDetails};

use crate::auth::password::{Admin, LoginRequest};

use super::category;
use super::inventory;
use super::pagination::{PaginatedResponse, PaginationQuery};
use super::product;
use super::sales;
use crate::auth::password;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "API to be consumed by user applications"),
    paths(
        product::handler::get_products_paginated,
        product::handler::get_product_by_id,
        product::handler::find_products,
    ),
    components(
        schemas(
            UserProduct,
            PaginationQuery,
            PaginatedResponse<UserProduct>,
            product::model::ProductFilter,
        )
    ),
    tags(
        (name = "Products")
    ),
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    info(description = "API to be consumed by admin dashboard"),
    paths(
        product::handler::get_all_products,
        product::handler::get_products_paginated_admin,
        product::handler::get_product_by_id_admin,
        product::handler::create_product,
        product::handler::update_product,
        product::handler::delete_product,
        product::handler::find_products_admin_advanced,
        product::handler::find_products_admin,
        password::admin_login_handler,
        password::admin_logout_handler,
        category::handler::create_category,
        category::handler::get_all_categories,
        category::handler::update_category,
        sales::handler::get_sales_paginated,
        sales::handler::get_sale_detail_by_id,
        inventory::handler::insert_inventory,
    
    ),
    components(
        schemas(
            LoginRequest,
            Admin,
            AdminProduct,
            PaginationQuery,
            CreateProductPayload,
            UpdateProductPayload,
            product::model::ProductFilter,
            PaginatedResponse<UserProduct>,
            PaginatedResponse<AdminProduct>,
            AdminCategory,
            CreateCategoryPayload,
            UpdateCategoryPayload,
            AdminSale,
            AdminSaleDetails,
            AdminInventory,
            NewInventoryPayload,
        )
    ),
    tags(
        (name = "Admin Products"),
        (name = "Admin Categories"),
        (name = "Admin Sales"),
        (name = "Admin Inventory"),
        (name = "Authentication"),
    )
)]
pub struct AdminApiDoc;
