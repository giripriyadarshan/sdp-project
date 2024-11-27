use async_graphql::{InputObject, SimpleObject};

#[derive(SimpleObject)]
pub struct Addresses {
    address_id: i64,
    address_type: String,
    city: String,
    country: String,
    customer_id: i64,
    is_default: bool,
    postal_code: String,
    state: String,
    street_address: String,
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
