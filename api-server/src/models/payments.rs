use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::Date;

#[derive(SimpleObject)]
pub struct PaymentMethods {
    pub payment_method_id: i32,
    pub customer_id: i32,
    pub payment_type: String,
    pub is_default: Option<bool>,
    pub bank_name: Option<String>,
    pub account_holder_name: Option<String>,
    pub card_number: Option<String>,
    pub card_expiration_date: Option<Date>,
    pub iban: Option<String>,
    pub upi_id: Option<String>,
    pub bank_account_number: Option<String>,
    pub ifsc_code: Option<String>,
    pub card_type_id: Option<i32>,
}

#[derive(InputObject)]
pub struct RegisterPaymentMethod {
    pub customer_id: i32,
    pub payment_type: String,
    pub is_default: Option<bool>,
    pub bank_name: Option<String>,
    pub account_holder_name: Option<String>,
    pub card_number: Option<String>,
    pub card_expiration_date: Option<Date>,
    pub iban: Option<String>,
    pub upi_id: Option<String>,
    pub bank_account_number: Option<String>,
    pub ifsc_code: Option<String>,
    pub card_type_id: Option<i32>,
}

pub struct CardTypes {
    pub card_type_id: i32,
    pub name: String,
}

#[derive(InputObject)]
pub struct RegisterCardType {
    pub name: String,
}
