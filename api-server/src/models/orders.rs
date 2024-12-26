use crate::entity::orders::Model as OrdersModel;
use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Orders {
    pub order_id: i32,
    pub customer_id: i32,
    pub order_date: Option<DateTimeWithTimeZone>,
    pub total_amount: f64,
    pub status: String,
    pub shipping_address_id: i32,
    pub payment_method_id: i32,
    pub discount_id: Option<i32>,
}

impl From<OrdersModel> for Orders {
    fn from(val: OrdersModel) -> Orders {
        Orders {
            order_id: val.order_id,
            customer_id: val.customer_id,
            order_date: val.order_date,
            total_amount: val.total_amount.to_string().parse::<f64>().unwrap(),
            status: val.status,
            shipping_address_id: val.shipping_address_id,
            payment_method_id: val.payment_method_id,
            discount_id: val.discount_id,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterOrder {
    pub shipping_address_id: i32,
    pub payment_method_id: i32,
    pub discount_code: Option<String>,
    pub order_items: Vec<RegisterOrderItem>,
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
    pub product_id: i32,
    pub quantity: i32,
}
