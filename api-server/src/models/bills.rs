use crate::entity::bills::Model as BillsModel;
use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Bills {
    bill_date: Option<DateTimeWithTimeZone>,
    bill_id: i32,
    order_id: i32,
    payment_status: String,
    total_amount: f64,
}

impl From<BillsModel> for Bills {
    fn from(val: BillsModel) -> Bills {
        Bills {
            bill_date: val.bill_date,
            bill_id: val.bill_id,
            payment_status: val.payment_status,
            total_amount: f64::try_from(val.total_amount).unwrap(),
            order_id: val.order_id,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterBill {
    bill_date: Option<DateTimeWithTimeZone>,
    order_id: i64,
    payment_status: String,
    total_amount: f64,
}
