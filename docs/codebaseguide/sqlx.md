## Query Limitations

SQLX sometimes has limitations on which columns can be null when there are SQL JOIN presents.
It can not properly infer nullability and therefore can't scan into Option values.
Use `?` or `!` when dealing with nullable columns.
Also list out all the columns by name instead of using `*`
