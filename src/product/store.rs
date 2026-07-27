use crate::app_error::StoreError;
use crate::db::DBQueryFilter;
use crate::product::model::{InsertProduct, ProductFilter};
use crate::product::model::{ProductRow, UpdateProduct};
use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<ProductRow>, StoreError> {
    let product_rows = sqlx::query_as!(
        ProductRow,
        r#"
            SELECT
                p.id as "id!",
                p.barcode as "barcode!",
                p.name as "name!",
                p.description,
                p.category_id,
                c1.name as category,
                p.sub_category_id,
                c2.name as sub_category,
                p.is_active as "is_active!",
                p.selling_price as "selling_price!",
                p.created_at as "created_at!",
                p.updated_at as "updated_at!",
                p.created_by as "created_by!",
                p.updated_by as "updated_by!",
                COALESCE((SELECT SUM(quantity_remaining)::INT4 FROM inventory WHERE product_id = p.id), 0) as "quantity_in_stock!",
                p.image_url
            FROM products p
            LEFT JOIN categories c1 on c1.id = p.category_id
            LEFT JOIN categories c2 on c2.id = p.sub_category_id
            "#
    )
        .fetch_all(pool)
        .await?;
    tracing::info!("total product rows: {}", product_rows.len());
    Ok(product_rows)
}

pub async fn fetch_paginated(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    is_active: bool,
) -> Result<(Vec<ProductRow>, i64), StoreError> {
    let products = sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.*,c1.name as category, c2.name as sub_category, COALESCE((SELECT SUM(quantity_remaining)::INT4 FROM inventory WHERE product_id = p.id), 0) AS "quantity_in_stock!"
        FROM products p
        JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        WHERE p.is_active = $3
        LIMIT $1 OFFSET $2"#,
        limit,
        offset,
        is_active,
    )
        .fetch_all(pool)
        .await?;

    let total_items = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM products WHERE is_active = $1",
        is_active
    )
        .fetch_one(pool)
        .await?;

    Ok((products, total_items.unwrap_or(0)))
}

pub async fn fetch_paginated_admin(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ProductRow>, i64), StoreError> {
    let products = sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.*,c1.name as category, c2.name as sub_category, COALESCE((SELECT SUM(quantity_remaining) FROM inventory WHERE product_id = p.id), 0)::INT4 as "quantity_in_stock!"
        FROM products p
        JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        LIMIT $1 OFFSET $2"#,
        limit,
        offset,
    )
        .fetch_all(pool)
        .await?;

    let total_items = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;

    Ok((products, total_items.unwrap_or(0)))
}

pub async fn fetch_by_id<'e, E>(executor: E, id: Uuid) -> Result<ProductRow, StoreError>
where
    E: Executor<'e, Database=Postgres>,
{
    sqlx::query_as!(
        ProductRow,
        r#"
        SELECT
            p.id as "id!",
            p.barcode as "barcode!",
            p.name as "name!",
            p.description,
            p.category_id,
            c1.name as category,
            p.sub_category_id,
            c2.name as sub_category,
            p.is_active as "is_active!",
            p.selling_price as "selling_price!",
            p.created_at as "created_at!",
            p.updated_at as "updated_at!",
            p.created_by as "created_by!",
            p.updated_by as "updated_by!",
            COALESCE((SELECT SUM(quantity_remaining) FROM inventory WHERE product_id = p.id), 0)::INT4 as "quantity_in_stock!",
            p.image_url
        FROM products p
        LEFT JOIN categories c1 on c1.id = p.category_id
        LEFT JOIN categories c2 on c2.id = p.sub_category_id
        WHERE p.id = $1"#,
        id
    )
        .fetch_one(executor)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StoreError::RecordNotFound,
            e => StoreError::Database(e),
        })
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    payload: InsertProduct,
) -> Result<Uuid, StoreError> {
    let product_id = sqlx::query_scalar!(
        r#"
        INSERT INTO products (barcode, name, description,category_id,sub_category_id,selling_price,is_active,image_url,created_by,updated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7,$8,$9,$9)
        RETURNING
            id
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category_id,
        payload.sub_category_id,
        payload.selling_price,
        payload.is_active,
        payload.image_url,
        payload.created_by
    )
        .fetch_one(&mut **tx)
        .await?;
    Ok(product_id)
}

pub async fn update(
    tx: &mut Transaction<'_, Postgres>,
    payload: UpdateProduct,
) -> Result<ProductRow, StoreError> {
    let product_id = sqlx::query_scalar!(
        r#"UPDATE products
       SET barcode = $1, name = $2, description = $3,category_id = $4,sub_category_id = $5, selling_price = $6, is_active = $7, image_url = $8, updated_by= $9, updated_at = NOW()
       WHERE id = $10
       RETURNING id
        "#,
        payload.barcode,
        payload.name,
        payload.description,
        payload.category_id,
        payload.sub_category_id,
        payload.selling_price,
        payload.is_active,
        payload.image_url,
        payload.updated_by,
        payload.id,
    )
        .fetch_one(&mut **tx)
        .await.map_err(StoreError::Database)?;
    fetch_by_id(&mut **tx, product_id).await
}

pub async fn delete(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<(), StoreError> {
    let result = sqlx::query!(r#"DELETE from products WHERE id = $1"#, id)
        .execute(&mut **tx)
        .await?;
    let affected = result.rows_affected();

    if affected > 1 {
        return Err(StoreError::UnexpectedRowsAffected(affected));
    }
    if affected == 0 {
        return Err(StoreError::RecordNotFound);
    }
    Ok(())
}

pub async fn fetch_products_filtered<F>(
    pool: &PgPool,
    filters: &F,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ProductRow>, i64), StoreError>
where
    F: DBQueryFilter<Postgres>,
{
    let mut data_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r"
            SELECT
                p.id,
                barcode,
                p.name,
                description,
                category_id,
                c1.name as category,
                sub_category_id,
                c2.name as sub_category,
                is_active,
                p.selling_price,
                p.created_at,
                p.updated_at,
                p.created_by,
                p.updated_by,
                COALESCE((SELECT SUM(quantity_remaining) FROM inventory WHERE product_id = p.id), 0)::INT4 AS quantity_in_stock,
                p.image_url
            FROM products p
            LEFT JOIN categories c1 on c1.id = p.category_id
            LEFT JOIN categories c2 on c2.id = p.sub_category_id
            WHERE 1=1",
    );

    filters.apply_filter(&mut data_builder);

    data_builder.push(" LIMIT ");
    data_builder.push_bind(limit);
    data_builder.push(" OFFSET ");
    data_builder.push_bind(offset);

    let query = data_builder.build_query_as::<ProductRow>();
    let product_rows = query.fetch_all(pool).await?;

    let mut count_qb = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM products p WHERE 1=1");
    filters.apply_filter(&mut count_qb);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    Ok((product_rows, total))
}

pub async fn fetch_products_search(
    pool: &PgPool,
    filter: ProductFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ProductRow>, i64), StoreError> {
    let search_term = format!("%{}%", filter.query);

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r"
                SELECT
                    p.id,
                    barcode,
                    p.name,
                    description,
                    category_id,
                    c1.name as category,
                    sub_category_id,
                    c2.name as sub_category,
                    is_active,
                    p.selling_price,
                    p.created_at,
                    p.updated_at,
                    p.created_by,
                    p.updated_by,
                    COALESCE((SELECT SUM(quantity_remaining) FROM inventory WHERE product_id = p.id), 0)::INT4 AS quantity_in_stock,
                    p.image_url
                FROM products p
                LEFT JOIN categories c1 on c1.id = p.category_id
                LEFT JOIN categories c2 on c2.id = p.sub_category_id
                WHERE 1=1
            ",
    );

    qb.push(" AND (p.name ILIKE ");
    qb.push_bind(search_term.clone());

    qb.push(" OR p.barcode ILIKE ");
    qb.push_bind(search_term.clone());
    qb.push(")");

    if let Some(category_id) = filter.category_id {
        qb.push(" AND p.category_id =");
        qb.push_bind(category_id);
    }
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    let product_query = qb.build_query_as::<ProductRow>();

    let result = product_query.fetch_all(pool).await?;

    let mut cqb: QueryBuilder<Postgres> =
        QueryBuilder::new("select COUNT(*) from products p where 1=1");

    cqb.push(" AND (p.name ILIKE ");
    cqb.push_bind(search_term.clone());

    cqb.push(" OR p.barcode ILIKE ");
    cqb.push_bind(search_term);
    cqb.push(")");

    if let Some(category_id) = filter.category_id {
        cqb.push(" AND p.category_id =");
        cqb.push_bind(category_id);
    }

    let count_query = cqb.build_query_scalar();
    let total_count: Option<i64> = count_query.fetch_one(pool).await?;

    Ok((result, total_count.unwrap_or(0)))
}
