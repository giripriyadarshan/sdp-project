use crate::entity::{payment_methods::Model as PaymentMethodsModel, card_types::Model as CardTypesModel};
use async_graphql::{InputObject, SimpleObject};
use sea_orm::{prelude::Date, ActiveEnum};

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

impl From<PaymentMethodsModel> for PaymentMethods {
    fn from(payment_method: PaymentMethodsModel) -> Self {
        Self {
            payment_method_id: payment_method.payment_method_id,
            customer_id: payment_method.customer_id,
            payment_type: payment_method.payment_type.to_value().to_string(),
            is_default: payment_method.is_default,
            bank_name: payment_method.bank_name,
            account_holder_name: payment_method.account_holder_name,
            card_number: payment_method.card_number,
            card_expiration_date: payment_method.card_expiration_date,
            iban: payment_method.iban,
            upi_id: payment_method.upi_id,
            bank_account_number: payment_method.bank_account_number,
            ifsc_code: payment_method.ifsc_code,
            card_type_id: payment_method.card_type_id,
        }
    }
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

#[derive(SimpleObject)]
pub struct CardTypes {
    pub card_type_id: i32,
    pub name: String,
}

impl From<CardTypesModel> for CardTypes {
    fn from(cardtypes: CardTypesModel) -> Self {
        Self {
            card_type_id: cardtypes.card_type_id,
            name: cardtypes.name
        }
    }
}

#[derive(InputObject)]
pub struct RegisterCardType {
    pub name: String,
}
