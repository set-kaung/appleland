-- Add migration script here
alter table products
add category text not null,
add sub_category text,
add is_active boolean not null;