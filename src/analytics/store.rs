use crate::analytics::model::{
    ByCategoryFilter, CategoryPerformanceRow, ProductPerformanceFilter, ProductPerformanceRow,
};
use crate::app_error::StoreError;
use crate::db::DBQueryFilter;
use sqlx::{PgPool, Postgres, QueryBuilder};

pub async fn get_product_performance(
    pool: &PgPool,
    filter: &ProductPerformanceFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ProductPerformanceRow>, i64), StoreError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
        SELECT
            p.id AS product_id,
            p.name AS name,
            c.name AS category,
            SUM(si.quantity) AS units_sold,
            SUM(si.quantity * si.selling_price) AS revenue,
            SUM(si.quantity * si.cost_price) AS cost,
            SUM(si.quantity * (si.selling_price - si.cost_price)) AS profit,
            CASE
                WHEN SUM(si.quantity * si.selling_price) = 0 THEN 0
                ELSE ROUND(SUM(si.quantity * (si.selling_price - si.cost_price)) / SUM(si.quantity * si.selling_price) * 100, 2)
            END AS margin_pct,
            ROUND(
                SUM(si.quantity * si.selling_price)
                / NULLIF(SUM(SUM(si.quantity * si.selling_price)) OVER (), 0) * 100,
            2) AS share_pct,
            MAX(s.created_at) AS last_sold_at
        FROM sale_items si
        JOIN sales s ON s.id = si.sale_id
        JOIN products p ON p.id = si.product_id
        LEFT JOIN categories c ON c.id = p.category_id AND c.is_deleted IS FALSE
        WHERE 1=1",
    );
    filter.apply_filter(&mut qb);
    qb.push(" GROUP BY p.id, p.name, c.name");
    filter.apply_sort(&mut qb);
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb.build_query_as::<ProductPerformanceRow>().fetch_all(pool).await?;

    let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COUNT(DISTINCT p.id) FROM sale_items si JOIN sales s ON s.id = si.sale_id JOIN products p ON p.id = si.product_id WHERE 1=1",
    );
    filter.apply_filter(&mut count_qb);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    Ok((rows, total))
}

pub async fn get_category_performance(
    pool: &PgPool,
    filter: &ByCategoryFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<CategoryPerformanceRow>, i64), StoreError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
        SELECT
            p.category_id AS category_id,
            c.name AS category,
            SUM(si.quantity) AS units_sold,
            SUM(si.quantity * si.selling_price) AS revenue,
            SUM(si.quantity * si.cost_price) AS cost,
            SUM(si.quantity * (si.selling_price - si.cost_price)) AS profit,
            CASE
                WHEN SUM(si.quantity * si.selling_price) = 0 THEN 0
                ELSE ROUND(SUM(si.quantity * (si.selling_price - si.cost_price)) / SUM(si.quantity * si.selling_price) * 100, 2)
            END AS margin_pct,
            ROUND(
                SUM(si.quantity * si.selling_price)
                / NULLIF(SUM(SUM(si.quantity * si.selling_price)) OVER (), 0) * 100,
            2) AS share_pct
        FROM sale_items si
        JOIN sales s ON s.id = si.sale_id
        JOIN products p ON p.id = si.product_id
        LEFT JOIN categories c ON c.id = p.category_id AND c.is_deleted IS FALSE
        WHERE 1=1",
    );
    filter.apply_filter(&mut qb);
    qb.push(" GROUP BY p.category_id, c.name");
    filter.apply_sort(&mut qb);
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb.build_query_as::<CategoryPerformanceRow>().fetch_all(pool).await?;

    let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COUNT(DISTINCT p.category_id) FROM sale_items si JOIN sales s ON s.id = si.sale_id JOIN products p ON p.id = si.product_id WHERE 1=1",
    );
    filter.apply_filter(&mut count_qb);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    Ok((rows, total))
}
