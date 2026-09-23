use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i64,
}

impl PaginationQuery {
    pub fn get_limit_and_offset(&self) -> (i64, i64, i64) {
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1).saturating_mul(page_size);

        (page_size, offset, page)
    }
}
