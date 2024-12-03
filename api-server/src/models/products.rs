use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Products {
    pub product_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub base_price: String,
    pub category_id: Option<i32>,
    pub supplier_id: Option<i32>,
    pub stock_quantity: i32,
    pub media_paths: Option<Vec<String>>,
    pub base_product_id: Option<i32>,
}

#[derive(InputObject)]
pub struct RegisterProduct {
    pub name: String,
    pub description: Option<String>,
    pub base_price: String,
    pub category_id: Option<i32>,
    pub supplier_id: Option<i32>,
    pub stock_quantity: i32,
    pub media_paths: Option<Vec<String>>,
    pub base_product_id: Option<i32>,
}

#[derive(SimpleObject)]
pub struct Categories {
    pub category_id: i32,
    pub name: String,
    pub parent_category_id: Option<i32>,
}

#[derive(InputObject)]
pub struct RegisterCategory {
    pub name: String,
    pub parent_category_id: Option<i32>,
}

#[derive(SimpleObject)]
pub struct Discounts {
    pub discount_id: i32,
    pub code: Option<String>,
    pub description: Option<String>,
    pub discount_value: i32,
    pub discount_type: String,
    pub valid_from: Option<DateTimeWithTimeZone>,
    pub valid_until: Option<DateTimeWithTimeZone>,
    pub max_uses: Option<i32>,
    pub times_used: Option<i32>,
    pub product_id: Option<i32>,
    pub category_id: Option<i32>,
    pub min_quantity: Option<i32>,
}

#[derive(InputObject)]
pub struct RegisterDiscount {
    pub code: Option<String>,
    pub description: Option<String>,
    pub discount_value: i32,
    pub discount_type: String,
    pub valid_from: Option<DateTimeWithTimeZone>,
    pub valid_until: Option<DateTimeWithTimeZone>,
    pub max_uses: Option<i32>,
    pub times_used: Option<i32>,
    pub product_id: Option<i32>,
    pub category_id: Option<i32>,
    pub min_quantity: Option<i32>,
}

#[derive(SimpleObject)]
pub struct Reviews {
    pub review_id: i32,
    pub customer_id: i32,
    pub product_id: i32,
    pub rating: Option<i32>,
    pub review_text: Option<String>,
    pub review_date: Option<DateTimeWithTimeZone>,
    pub media_paths: Option<Vec<String>>,
}

#[derive(InputObject)]
pub struct RegisterReview {
    pub customer_id: i32,
    pub product_id: i32,
    pub rating: Option<i32>,
    pub review_text: Option<String>,
    pub review_date: Option<DateTimeWithTimeZone>,
    pub media_paths: Option<Vec<String>>,
}
