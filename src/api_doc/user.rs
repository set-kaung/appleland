use crate::product::model::UserProduct;
use crate::user::model::CreateUserPayload;

use super::super::pagination::{PaginatedResponse, PaginationQuery};
use super::BearerSecurityAddon;
use crate::{playground, product, user};
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
        playground::handler::claim_token,
        playground::handler::get_user_tickets,
        playground::handler::claim_free_ticket,
        user::handler::create_user,
    ),
    components(
        schemas(
            UserProduct,
            PaginationQuery,
            PaginatedResponse<UserProduct>,
            product::model::ProductFilter,
            playground::model::ClaimTokenPayload,
            playground::model::UserPlaygroundTickets,
            playground::model::UserPlaygroundPurchase,
            playground::model::UserPlaygroundPurchaseResponse,
            playground::model::UserPlaygroundAccount,
            CreateUserPayload
        )
    ),
    tags(
        (name = "Products", description = "Browse and search the product catalog"),
        (name = "Playground", description = "Check playground tickets,eligibility, claim"),
        (name = "User", description = "User account management")
    ),
)]
pub struct ApiDoc;
