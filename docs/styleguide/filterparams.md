# DB Fitlers

The db filters should be a struct and implement the DBFilter trait. See example.

```rs
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
```
