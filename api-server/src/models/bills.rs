use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Bills {
    bill_date: Option<DateTimeWithTimeZone>,
    bill_id: i64,
    order_id: i64,
    payment_status: String,
    total_amount: f64,
}

#[derive(InputObject)]
pub struct RegisterBill {
    bill_date: Option<DateTimeWithTimeZone>,
    order_id: i64,
    payment_status: String,
    total_amount: f64,
}
