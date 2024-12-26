use crate::entity::{
    card_types::{self, Model as CardTypesModel},
    payment_methods::{self, Model as PaymentMethodsModel},
    sea_orm_active_enums::PaymentMethodType,
};
use async_graphql::{InputObject, SimpleObject};
use sea_orm::{
    prelude::Date, ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    DatabaseTransaction, EntityTrait, QueryFilter,
};

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
    pub card_type_name: Option<String>,
}

pub async fn create_payment_method(
    customer_id: i32,
    is_default: Option<bool>,
    input: RegisterPaymentMethod,
    txn: &DatabaseTransaction,
) -> Result<payment_methods::ActiveModel, async_graphql::Error> {
    // check if any default payment method exists and update it to not default
    if is_default.unwrap_or(false) {
        let default_payment_method = payment_methods::Entity::find()
            .filter(payment_methods::Column::CustomerId.eq(customer_id))
            .filter(payment_methods::Column::IsDefault.eq(true))
            .one(txn)
            .await?;
        if default_payment_method.is_some() {
            // update the existing default payment method to not default
            let mut default_payment_method: payment_methods::ActiveModel =
                default_payment_method.unwrap().into();
            default_payment_method.is_default = Set(Some(false));
            default_payment_method.update(txn).await?;
        }
    }

    match input.payment_type.as_str() {
        "card" => {
            let card_type_id = Some(
                card_types::Entity::insert(card_types::ActiveModel {
                    name: Set(input
                        .card_type_name
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string())),
                    ..Default::default()
                })
                .exec(txn)
                .await?
                .last_insert_id,
            );
            Ok(payment_methods::ActiveModel {
                customer_id: Set(customer_id),
                payment_type: Set(PaymentMethodType::Card),
                is_default: Set(is_default),
                card_number: Set(Some(
                    input.card_number.clone().ok_or("Card number is required")?,
                )),
                card_expiration_date: Set(Some(
                    input
                        .card_expiration_date
                        .ok_or("Card expiration date is required")?,
                )),
                card_type_id: Set(card_type_id),
                ..Default::default()
            })
        }
        "upi" => Ok(payment_methods::ActiveModel {
            customer_id: Set(customer_id),
            payment_type: Set(PaymentMethodType::Upi),
            is_default: Set(is_default),
            upi_id: Set(Some(input.upi_id.clone().ok_or("UPI ID is required")?)),
            ..Default::default()
        }),
        "iban" => Ok(payment_methods::ActiveModel {
            customer_id: Set(customer_id),
            payment_type: Set(PaymentMethodType::Iban),
            is_default: Set(is_default),
            iban: Set(Some(input.iban.clone().ok_or("IBAN number is required")?)),
            ..Default::default()
        }),
        "netbanking" => Ok(payment_methods::ActiveModel {
            customer_id: Set(customer_id),
            payment_type: Set(PaymentMethodType::Netbanking),
            is_default: Set(is_default),
            bank_name: Set(Some(
                input.bank_name.clone().ok_or("Bank name is required")?,
            )),
            account_holder_name: Set(Some(
                input
                    .account_holder_name
                    .clone()
                    .ok_or("Account holder name is required")?,
            )),
            bank_account_number: Set(Some(
                input
                    .bank_account_number
                    .clone()
                    .ok_or("Bank account number is required")?,
            )),
            ifsc_code: Set(Some(
                input.ifsc_code.clone().ok_or("IFSC code is required")?,
            )),
            ..Default::default()
        }),
        _ => Err("Invalid payment type".into()),
    }
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
            name: cardtypes.name,
        }
    }
}
