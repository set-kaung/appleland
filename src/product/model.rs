use crate::app_error::APIError;
use crate::db::{DBQueryFilter, escape_like};
use crate::inventory::model::NewInventoryPayload;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AdminProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category: Option<String>,
    pub sub_category_id: Option<Uuid>,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub selling_price: Decimal,
    pub quantity_in_stock: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub is_perishable: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub selling_price: Decimal,
    pub quantity_in_stock: i32,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductPayload {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub selling_price: Decimal,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub inventory: Option<NewInventoryPayload>,
    pub is_perishable: bool,
}

impl CreateProductPayload {
    pub fn validate(&self) -> Result<(), APIError> {
        if self.name.trim().is_empty() {
            return Err(APIError::BadRequest(
                "product name must not be empty".to_string(),
            ));
        }
        if self.barcode.trim().is_empty() {
            return Err(APIError::BadRequest(
                "barcode must not be empty".to_string(),
            ));
        }
        if self.selling_price < Decimal::ZERO {
            return Err(APIError::BadRequest(
                "selling price must not be negative".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct InsertProduct {
    pub name: String,
    pub barcode: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub selling_price: Decimal,
    pub image_url: Option<String>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub is_perishable: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductPayload {
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub selling_price: Decimal,
    pub image_url: Option<String>,
    pub is_perishable: bool,
}

impl UpdateProductPayload {
    pub fn validate(&self) -> Result<(), APIError> {
        if self.name.trim().is_empty() {
            return Err(APIError::BadRequest(
                "product name must not be empty".to_string(),
            ));
        }
        if self.barcode.trim().is_empty() {
            return Err(APIError::BadRequest(
                "barcode must not be empty".to_string(),
            ));
        }
        if self.selling_price < Decimal::ZERO {
            return Err(APIError::BadRequest(
                "selling price must not be negative".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub sub_category_id: Option<Uuid>,
    pub is_active: bool,
    pub selling_price: Decimal,
    pub updated_by: Uuid,
    pub image_url: Option<String>,
    pub is_perishable: bool,
}

#[derive(Debug, FromRow)]
pub struct ProductRow {
    pub id: Uuid,
    pub barcode: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category: Option<String>,
    pub sub_category_id: Option<Uuid>,
    pub sub_category: Option<String>,
    pub is_active: bool,
    pub selling_price: Decimal,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub quantity_in_stock: i32,
    pub image_url: Option<String>,
    pub is_perishable: bool,
}

impl From<ProductRow> for UserProduct {
    fn from(row: ProductRow) -> Self {
        Self {
            id: row.id,
            barcode: row.barcode,
            name: row.name,
            description: row.description,
            category: row.category,
            sub_category: row.sub_category,
            selling_price: row.selling_price,
            quantity_in_stock: row.quantity_in_stock,
            image_url: row.image_url,
        }
    }
}

impl From<ProductRow> for AdminProduct {
    fn from(row: ProductRow) -> Self {
        Self {
            id: row.id,
            barcode: row.barcode,
            name: row.name,
            description: row.description,
            category_id: row.category_id,
            category: row.category,
            sub_category_id: row.sub_category_id,
            sub_category: row.sub_category,
            is_active: row.is_active,
            selling_price: row.selling_price,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
            quantity_in_stock: row.quantity_in_stock,
            image_url: row.image_url,
            is_perishable: row.is_perishable,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProductSearchParams {
    pub name: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub categories_id: Option<Vec<Uuid>>,
    pub sub_categories_id: Option<Vec<Uuid>>,
    pub barcode: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProductSearchParamsAdmin {
    #[serde(flatten)]
    pub common: ProductSearchParams,
    // exists to provide admin to filter between active and inactive items
    // not intended for user facing searches
    pub is_active: Option<bool>,
}

impl DBQueryFilter<Postgres> for ProductSearchParams {
    fn apply_filter(&self, query_builder: &mut QueryBuilder<Postgres>) {
        if let Some(name) = &self.name {
            query_builder.push(" AND p.name ILIKE ");
            query_builder.push_bind(format!("%{}%", escape_like(name)));
        }

        if let Some(barcode) = &self.barcode {
            query_builder.push(" AND p.barcode ILIKE ");
            query_builder.push_bind(format!("%{}%", escape_like(barcode)));
        }

        if let Some(categories_id) = &self.categories_id {
            query_builder.push(" AND p.category_id = ANY(");
            query_builder.push_bind(categories_id.clone());
            query_builder.push(")");
        }

        if let Some(max_price) = &self.max_price {
            query_builder.push(" AND p.selling_price <=");
            query_builder.push_bind(*max_price);
        }

        if let Some(min_price) = &self.min_price {
            query_builder.push(" AND p.selling_price >=");
            query_builder.push_bind(*min_price);
        }
        if let Some(sub_categories_id) = &self.sub_categories_id {
            query_builder.push(" AND p.sub_category_id = ANY(");
            query_builder.push_bind(sub_categories_id.clone());
            query_builder.push(")");
        }
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ProductFilter {
    pub query: String,
    pub category_id: Option<Uuid>,
}

impl DBQueryFilter<Postgres> for ProductSearchParamsAdmin {
    fn apply_filter(&self, query_builder: &mut QueryBuilder<Postgres>) {
        self.common.apply_filter(query_builder);
        if let Some(active) = self.is_active {
            query_builder.push(" AND p.is_active = ");
            query_builder.push_bind(active);
        }
    }
}
