use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct ShoppingCarts {
    pub cart_id: i32,
    pub customer_id: i32,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(InputObject)]
pub struct RegisterShoppingCart {
    pub customer_id: i32,
}

#[derive(SimpleObject)]
pub struct CartItems {
    pub cart_item_id: i32,
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(InputObject)]
pub struct RegisterCartItem {
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}
