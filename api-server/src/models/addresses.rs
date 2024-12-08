use crate::entity::addresses::Model as AddressModel;
use async_graphql::{InputObject, SimpleObject};

#[derive(SimpleObject)]
pub struct Addresses {
    address_id: i32,
    address_type_id: Option<i32>,
    city: String,
    country: String,
    customer_id: i32,
    is_default: Option<bool>,
    postal_code: String,
    state: Option<String>,
    street_address: String,
}

impl From<AddressModel> for Addresses {
    fn from(address: AddressModel) -> Self {
        Self {
            address_id: address.address_id,
            address_type_id: address.address_type_id,
            city: address.city,
            country: address.country,
            customer_id: address.customer_id,
            is_default: address.is_default,
            postal_code: address.postal_code,
            state: address.state,
            street_address: address.street_address,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterAddress {
    address_type: String,
    city: String,
    country: String,
    customer_id: i64,
    is_default: bool,
    postal_code: String,
    state: String,
    street_address: String,
}

#[derive(SimpleObject)]
pub struct AddressType {
    address_type_id: i64,
    name: String,
}

#[derive(InputObject)]
pub struct RegisterAddressType {
    name: String,
}
