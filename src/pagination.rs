use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
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
        let per_page = self.per_page.unwrap_or(10).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1) * per_page;

        (per_page, offset, page)
    }
}
