use crate::entity::address_types::Model as AddressTypesModel;
use crate::entity::addresses::{self, Model as AddressModel};
use async_graphql::{InputObject, SimpleObject};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

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
    pub address_type: String,
    pub city: String,
    pub country: String,
    pub customer_id: i64,
    pub is_default: bool,
    pub postal_code: String,
    pub state: String,
    pub street_address: String,
}

impl RegisterAddress {
    pub(crate) fn clone(&self) -> RegisterAddress {
        RegisterAddress {
            address_type: self.address_type.clone(),
            city: self.city.clone(),
            country: self.country.clone(),
            customer_id: self.customer_id,
            is_default: self.is_default,
            postal_code: self.postal_code.clone(),
            state: self.state.clone(),
            street_address: self.street_address.clone(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AddressType {
    address_type_id: i32,
    name: String,
}

impl From<AddressTypesModel> for AddressType {
    fn from(address_type: AddressTypesModel) -> Self {
        Self {
            address_type_id: address_type.address_type_id,
            name: address_type.name,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterAddressType {
    pub name: String,
}

pub async fn create_address(
    input: RegisterAddress,
    customer_id: i32,
    address_type_id: i32,
    txn: &sea_orm::DatabaseTransaction,
) -> Result<addresses::ActiveModel, async_graphql::Error> {
    //check if default address exists and make it not default
    if input.is_default {
        let default_address = addresses::Entity::find()
            .filter(addresses::Column::CustomerId.eq(customer_id))
            .filter(addresses::Column::IsDefault.eq(true))
            .one(txn)
            .await?;
        if default_address.is_some() {
            let mut default_address: addresses::ActiveModel = default_address.unwrap().into();
            default_address.is_default = Set(Some(false));
            default_address.update(txn).await?;
        }
    }

    Ok(addresses::ActiveModel {
        customer_id: Set(customer_id),
        address_type_id: Set(Some(address_type_id)),
        street_address: Set(input.street_address),
        city: Set(input.city),
        state: Set(Some(input.state)),
        country: Set(input.country),
        postal_code: Set(input.postal_code),
        is_default: Set(Some(input.is_default)),
        ..Default::default()
    })
}
