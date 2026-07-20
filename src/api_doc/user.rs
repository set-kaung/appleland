use crate::product::model::UserProduct;

use super::super::pagination::{PaginatedResponse, PaginationQuery};
use super::super::product;
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
