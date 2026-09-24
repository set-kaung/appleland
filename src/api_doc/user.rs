use crate::orders::model::{
    UserOrderDetail, UserOrderHistoryResponse, UserOrderItem, UserOrderStatusEntry,
};
use crate::product::model::UserProduct;
use crate::sales::model::CreateUserSalePayload;
use crate::users::model::CreateUserPayload;

use super::super::pagination::{PaginatedResponse, PaginationQuery};
use super::BearerSecurityAddon;
use crate::{orders, playground, product, sales, users};
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
        playground::handler::get_checkout_data,
        users::handler::create_user,
        sales::handler::create_user_sale,
        orders::handler::get_user_orders_paginated,
        orders::handler::get_user_order_detail,
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
            playground::model::PlaygroundCheckoutData,
            CreateUserPayload,
            CreateUserSalePayload,
            sales::model::SaleProduct,
            UserOrderHistoryResponse,
            PaginatedResponse<UserOrderHistoryResponse>,
            UserOrderDetail,
            UserOrderItem,
            UserOrderStatusEntry,
            orders::model::OrderShippingAddress,
        )
    ),
    tags(
        (name = "Products", description = "Browse and search the product catalog"),
        (name = "User Playground", description = "Playground tickets, eligibility, claim"),
        (name = "User", description = "User account management"),
        (name = "User Sales", description = "Checkout and sales for users"),
        (name = "User Orders", description = "Order history and delivery tracking for users"),
    )
)]
pub struct ApiDoc;
