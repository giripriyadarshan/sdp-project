use crate::auth::auth::Auth;
use crate::entity::{
    categories::Model as CategoriesModel, products::Model as ProductsModel,
    reviews::Model as ReviewsModel,
};
use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::error::Error;
use sea_orm::ActiveValue::Set;
use std::string::ToString;

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

impl From<ProductsModel> for Products {
    fn from(val: ProductsModel) -> Products {
        Products {
            product_id: val.product_id,
            name: val.name,
            description: val.description,
            base_price: val.base_price.to_string(),
            category_id: val.category_id,
            supplier_id: val.supplier_id,
            stock_quantity: val.stock_quantity,
            media_paths: val.media_paths,
            base_product_id: val.base_product_id,
        }
    }
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

pub fn create_product_model(
    input: RegisterProduct,
    token: String,
) -> Result<crate::entity::products::ActiveModel, Error> {
    use crate::entity::products;
    Ok(products::ActiveModel {
        name: Set(input.name.clone()),
        description: Set(input.description.clone()),
        base_price: Set(input.base_price.parse::<i64>().unwrap().into()),
        supplier_id: Set(Some(
            Auth::verify_token(&token)
                .unwrap()
                .user_id
                .parse::<i32>()
                .unwrap(),
        )),
        ..Default::default()
    })
}

#[derive(SimpleObject)]
pub struct Categories {
    pub category_id: i32,
    pub name: String,
    pub parent_category_id: Option<i32>,
}

impl From<CategoriesModel> for Categories {
    fn from(val: CategoriesModel) -> Categories {
        Categories {
            category_id: val.category_id,
            name: val.name,
            parent_category_id: val.parent_category_id,
        }
    }
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

impl From<ReviewsModel> for Reviews {
    fn from(val: ReviewsModel) -> Reviews {
        Reviews {
            review_id: val.review_id,
            customer_id: val.customer_id,
            product_id: val.product_id,
            rating: val.rating,
            review_text: val.review_text,
            review_date: val.review_date,
            media_paths: val.media_paths,
        }
    }
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
