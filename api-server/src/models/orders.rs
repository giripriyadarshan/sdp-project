use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Orders {
    pub order_id: i32,
    pub customer_id: i32,
    pub order_date: Option<DateTimeWithTimeZone>,
    pub total_amount: i32,
    pub status: String,
    pub shipping_address_id: i32,
    pub payment_method_id: i32,
    pub discount_code_id: Option<i32>,
    pub discount_id: Option<i32>,
}

#[derive(InputObject)]
pub struct RegisterOrder {
    pub customer_id: i32,
    pub order_date: Option<DateTimeWithTimeZone>,
    pub total_amount: i32,
    pub status: String,
    pub shipping_address_id: i32,
    pub payment_method_id: i32,
    pub discount_code_id: Option<i32>,
    pub discount_id: Option<i32>,
}

#[derive(SimpleObject)]
pub struct OrderItems {
    pub order_item_id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
    pub discount_amount: i32,
}

#[derive(InputObject)]
pub struct RegisterOrderItem {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
    pub discount_amount: i32,
}
