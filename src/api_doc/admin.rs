use crate::product::model::{
    AdminProduct, CreateProductPayload, ProductFilter, ProductSearchParamsAdmin,
    UpdateProductPayload,
};

use crate::category::model::{
    AdminCategory, CreateCategoryPayload, DeleteCategoryResponse, UpdateCategoryPayload,
};
use crate::inventory::model::{AdminInventory, NewInventoryPayload};
use crate::orders::model::{UpdateOrderDeliveryStatusResponse, UpdateOrderTrackingURLPayload};
use crate::sales::model::{
    AdminSale, AdminSaleDetails, CreateAdminSalePayload, SaleItem, SaleProduct, SaleSummary,
    SaleSummaryFilter,
};

use crate::playground::model::CreateClaimTokenPayload;
use crate::staff::model::AdminStaff;

use crate::auth::admin as admin_auth;
use crate::category;
use crate::image_storage;
use crate::inventory;
use crate::orders;
use crate::pagination::{PaginatedResponse, PaginationQuery};
use crate::playground;
use crate::product;
use crate::sales;
use crate::staff;

use super::SessionCookieSecurityAddon;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Appleland Admin API",
        description = "API for the admin dashboard. Endpoints require an authenticated staff session established via the login endpoint (session cookie named `id`)."
    ),
    modifiers(&SessionCookieSecurityAddon),
    paths(
        category::handler::create_category,
        category::handler::get_all_categories,
        category::handler::update_category,
        category::handler::delete_category,
        inventory::handler::insert_inventory_record,
        inventory::handler::get_product_inventory_records,
        admin_auth::handler::admin_login_handler,
        admin_auth::handler::admin_logout_handler,
        playground::handler::creat_playground_token,
        product::handler::get_all_products,
        product::handler::get_products_paginated_admin,
        product::handler::get_product_by_id_admin,
        product::handler::create_product,
        product::handler::update_product,
        product::handler::delete_product,
        product::handler::search_products_admin_advanced,
        product::handler::search_products_admin,
        sales::handler::get_sales_paginated,
        sales::handler::get_sale_detail_by_id,
        sales::handler::create_admin_sale,
        sales::handler::sale_summary,
        orders::handler::update_order_tracking_url_handler,
        orders::handler::update_order_delivery_status,
        staff::handler::get_all_admin_staff,
        image_storage::upload_image,
    ),
    components(
        schemas(
            admin_auth::model::LoginRequest,
            admin_auth::model::Admin,
            AdminProduct,
            PaginationQuery,
            CreateProductPayload,
            UpdateProductPayload,
            ProductFilter,
            AdminCategory,
            CreateCategoryPayload,
            UpdateCategoryPayload,
            DeleteCategoryResponse,
            AdminSale,
            AdminSaleDetails,
            CreateAdminSalePayload,
            SaleItem,
            SaleProduct,
            SaleSummary,
            SaleSummaryFilter,
            ProductSearchParamsAdmin,
            PaginatedResponse<AdminProduct>,
            PaginatedResponse<AdminInventory>,
            AdminInventory,
            NewInventoryPayload,
            CreateClaimTokenPayload,
            UpdateOrderDeliveryStatusResponse,
            UpdateOrderTrackingURLPayload,
            AdminStaff,
            image_storage::UploadResponse
        )
    ),
    tags(
        (name = "Admin Products", description = "Product CRUD and search"),
        (name = "Admin Categories", description = "Product category management"),
        (name = "Admin Sales", description = "Sales records, checkout, and reporting"),
        (name = "Admin Inventory", description = "Stock replenishment"),
        (name = "Authentication", description = "Admin login and logout"),
        (name = "Uploads", description = "File upload utilities"),
    )
)]
pub struct AdminApiDoc;
