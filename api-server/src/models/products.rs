use crate::{
    entity::{
        categories::Model as CategoriesModel, discounts::Model as DiscountsModel, products,
        products::Entity as ProductsEntity, products::Model as ProductsModel,
        reviews::Model as ReviewsModel,
    },
    models::order_und_pagination::{OrderAndPagination, OrderByColumn, OrderByOrder, PageInfo},
};
use async_graphql::{InputObject, SimpleObject};
use sea_orm::{
    prelude::DateTimeWithTimeZone, sea_query::error::Error, ActiveValue::Set, ColumnTrait,
    DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Select,
};
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

#[derive(SimpleObject)]
pub struct ProductsPaginate {
    pub products: Vec<Products>,
    pub page_info: PageInfo,
}

pub async fn check_product_exists(
    txn: &DatabaseTransaction,
    product_id: i32,
) -> Result<(), async_graphql::Error> {
    use crate::entity::products;
    if products::Entity::find_by_id(product_id)
        .one(txn)
        .await?
        .is_none()
    {
        return Err("Product not found".into());
    }
    Ok(())
}

pub async fn paginate_products(
    paginator: OrderAndPagination,
    entity: Select<ProductsEntity>,
) -> Result<Select<ProductsEntity>, async_graphql::Error> {
    let order_by = paginator.order_by.column;
    let order = paginator.order_by.order;

    match (order, order_by) {
        (OrderByOrder::Asc, OrderByColumn::Date) => {
            Ok(entity.order_by_asc(products::Column::CreatedAt))
        }
        (OrderByOrder::Desc, OrderByColumn::Date) => {
            Ok(entity.order_by_desc(products::Column::CreatedAt))
        }
        (OrderByOrder::Asc, OrderByColumn::Amount) => {
            Ok(entity.order_by_asc(products::Column::BasePrice))
        }
        (OrderByOrder::Desc, OrderByColumn::Amount) => {
            Ok(entity.order_by_desc(products::Column::BasePrice))
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
    supplier_id: i32,
) -> Result<products::ActiveModel, Error> {
    use crate::entity::products;
    Ok(products::ActiveModel {
        name: Set(input.name.clone()),
        description: Set(input.description.clone()),
        base_price: Set(input.base_price.parse::<f32>().unwrap().try_into().unwrap()),
        supplier_id: Set(Some(supplier_id)),
        category_id: Set(input.category_id),
        base_product_id: Set(input.base_product_id),
        media_paths: Set(input.media_paths),
        stock_quantity: Set(input.stock_quantity),
        ..Default::default()
    })
}

pub async fn check_if_supplier_owns_product(
    txn: &DatabaseConnection,
    supplier_id: i32,
    product_id: i32,
) -> Result<(), async_graphql::Error> {
    use crate::entity::products;
    if products::Entity::find_by_id(product_id)
        .filter(products::Column::SupplierId.eq(supplier_id))
        .one(txn)
        .await?
        .is_none()
    {
        return Err("Supplier does not own this product".into());
    }
    Ok(())
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
    pub discount_value: f32,
    pub discount_type: String,
    pub valid_from: Option<DateTimeWithTimeZone>,
    pub valid_until: Option<DateTimeWithTimeZone>,
    pub max_uses: Option<i32>,
    pub times_used: Option<i32>,
    pub product_id: Option<i32>,
    pub category_id: Option<i32>,
    pub min_quantity: Option<i32>,
}

impl From<DiscountsModel> for Discounts {
    fn from(val: DiscountsModel) -> Discounts {
        Discounts {
            discount_id: val.discount_id,
            code: val.code,
            description: val.description,
            discount_value: f32::try_from(val.discount_value).unwrap(),
            discount_type: val.discount_type,
            valid_from: val.valid_from,
            valid_until: val.valid_until,
            max_uses: val.max_uses,
            times_used: val.times_used,
            product_id: val.product_id,
            category_id: val.category_id,
            min_quantity: val.min_quantity,
        }
    }
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
    pub product_id: i32,
    pub category_id: Option<i32>,
    pub min_quantity: Option<i32>,
}

pub fn create_discount_model(
    input: RegisterDiscount,
) -> Result<crate::entity::discounts::ActiveModel, Error> {
    use crate::entity::discounts;
    Ok(discounts::ActiveModel {
        code: Set(input.code),
        description: Set(input.description),
        discount_value: Set(input.discount_value.into()),
        discount_type: Set(input.discount_type),
        valid_from: Set(input.valid_from),
        valid_until: Set(input.valid_until),
        max_uses: Set(input.max_uses),
        times_used: Set(input.times_used),
        product_id: Set(Some(input.product_id)),
        category_id: Set(input.category_id),
        min_quantity: Set(input.min_quantity),
        ..Default::default()
    })
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

#[derive(SimpleObject)]
pub struct ReviewsPaginate {
    pub reviews: Vec<Reviews>,
    pub page_info: PageInfo,
}

#[derive(InputObject)]
pub struct RegisterReview {
    pub product_id: i32,
    pub rating: Option<i32>,
    pub review_text: Option<String>,
    pub review_date: Option<DateTimeWithTimeZone>,
    pub media_paths: Option<Vec<String>>,
}

pub fn create_review_model(
    input: RegisterReview,
    customer_id: i32,
) -> Result<crate::entity::reviews::ActiveModel, Error> {
    use crate::entity::reviews;
    Ok(reviews::ActiveModel {
        customer_id: Set(customer_id),
        product_id: Set(input.product_id),
        rating: Set(input.rating),
        review_text: Set(input.review_text),
        review_date: Set(input.review_date),
        media_paths: Set(input.media_paths),
        ..Default::default()
    })
}
