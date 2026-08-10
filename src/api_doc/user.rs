use crate::product::model::UserProduct;

use super::super::pagination::{PaginatedResponse, PaginationQuery};
use super::super::product;
use super::BearerSecurityAddon;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Appleland User API",
        description = "Public API for storefront/user applications. All endpoints require a valid Supabase JWT supplied via the `Authorization: Bearer <token>` header."
    ),
    modifiers(&BearerSecurityAddon),
    paths(
        product::handler::get_products_paginated,
        product::handler::get_product_by_id,
        product::handler::search_products,
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
        (name = "Products", description = "Browse and search the product catalog")
    ),
)]
pub struct ApiDoc;
