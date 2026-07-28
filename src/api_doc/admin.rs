use crate::product::model::{
    AdminProduct, CreateProductPayload, ProductSearchParamsAdmin, UpdateProductPayload,
};

use crate::category::model::{AdminCategory, CreateCategoryPayload, UpdateCategoryPayload};
use crate::inventory::model::{AdminInventory, NewInventoryPayload};
use crate::sales::model::{
    AdminSale, AdminSaleDetails, CreateSalePayload, SaleItem, SaleProduct, SaleSummary,
    SaleSummaryFilter,
};

use crate::auth::password::{Admin, LoginRequest};

use super::super::category;
use super::super::inventory;
use super::super::pagination::{PaginatedResponse, PaginationQuery};
use super::super::product;
use super::super::sales;
use crate::auth::password;
use utoipa::OpenApi;

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
        product::handler::search_products_admin_advanced,
        product::handler::search_products_admin,
        password::admin_login_handler,
        password::admin_logout_handler,
        category::handler::create_category,
        category::handler::get_all_categories,
        category::handler::update_category,
        sales::handler::get_sales_paginated,
        sales::handler::get_sale_detail_by_id,
        sales::handler::create_sale,
        sales::handler::sale_summary,
        inventory::handler::create_inventory,
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
            PaginatedResponse<AdminProduct>,
            AdminCategory,
            CreateCategoryPayload,
            UpdateCategoryPayload,
            AdminSale,
            AdminSaleDetails,
            CreateSalePayload,
            SaleItem,
            SaleProduct,
            SaleSummary,
            SaleSummaryFilter,
            ProductSearchParamsAdmin,
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
