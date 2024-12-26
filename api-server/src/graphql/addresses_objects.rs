use crate::models::addresses::AddressType;
use crate::{
    auth::{Auth, RoleGuard, ROLE_CUSTOMER},
    graphql::macros::role_guard,
    models::{
        addresses::{create_address, Addresses, RegisterAddress},
        user::get_customer_supplier_id,
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DbBackend, EntityTrait, ModelTrait, QueryFilter, Statement, TransactionTrait,
};

#[derive(Default)]
pub struct AddressesQuery;

#[derive(Default)]
pub struct AddressesMutation;

#[Object]
impl AddressesQuery {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn addresses(&self, ctx: &Context<'_>) -> Result<Vec<Addresses>, async_graphql::Error> {
        use crate::entity::addresses;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
        // this query includes inner join with users, customers and addresses tables
        // this query can be written entirely with SELECT and WHERE.
        // Basically, get customer_id using user_id in customer table and then insert it into addresses table.
        // But this has the keyword "JOIN" in it, so we'll go with this one.
        // For reference:
        // SELECT * FROM addresses WHERE customer_id = (SELECT customer_id FROM customers WHERE user_id = $1);
        let address = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT addresses.* FROM users
                    JOIN customers ON users.user_id = customers.user_id
                    JOIN addresses ON customers.customer_id = addresses.customer_id
                    WHERE users.user_id = $1;
                    ",
                vec![user_id.into()],
            ))
            .await?;

        let addresses: Vec<Addresses> = address
            .into_iter()
            .map(|item| {
                addresses::Model {
                    address_id: item.try_get::<i32>("", "address_id").unwrap().to_owned(),
                    address_type_id: item
                        .try_get::<Option<i32>>("", "address_type_id")
                        .unwrap()
                        .to_owned(),
                    city: item.try_get::<String>("", "city").unwrap().to_owned(),
                    country: item.try_get::<String>("", "country").unwrap().to_owned(),
                    customer_id: item.try_get::<i32>("", "customer_id").unwrap().to_owned(),
                    is_default: item
                        .try_get::<Option<bool>>("", "is_default")
                        .unwrap()
                        .to_owned(),
                    postal_code: item
                        .try_get::<String>("", "postal_code")
                        .unwrap()
                        .to_owned(),
                    state: item
                        .try_get::<Option<String>>("", "state")
                        .unwrap()
                        .to_owned(),
                    street_address: item
                        .try_get::<String>("", "street_address")
                        .unwrap()
                        .to_owned(),
                }
                .into()
            })
            .collect();

        Ok(addresses)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn address_type(
        &self,
        ctx: &Context<'_>,
        address_type_id: i32,
    ) -> Result<AddressType, async_graphql::Error> {
        use crate::entity::prelude::AddressTypes as AddressTypesEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let address_type = AddressTypesEntity::find_by_id(address_type_id)
            .one(db)
            .await?
            .ok_or("Address type not found")?;

        Ok(address_type.into())
    }
}

#[Object]
impl AddressesMutation {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_address(
        &self,
        ctx: &Context<'_>,
        input: RegisterAddress,
    ) -> Result<Addresses, async_graphql::Error> {
        use crate::entity::{
            address_types,
            prelude::{AddressTypes as AddressTypesEntity, Addresses as AddressesEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let address_type = address_types::ActiveModel {
            name: Set(input.clone().address_type),
            ..Default::default()
        };

        let address_type_id = AddressTypesEntity::insert(address_type)
            .exec(&txn)
            .await?
            .last_insert_id;

        let address = create_address(input, customer_id, address_type_id, &txn).await?;

        let insert_address = AddressesEntity::insert(address)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(insert_address.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_address(
        &self,
        ctx: &Context<'_>,
        address_id: i32,
        address_type_id: i32,
        input: RegisterAddress,
    ) -> Result<Addresses, async_graphql::Error> {
        use crate::entity::{addresses, prelude::Addresses as AddressesEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let mut address = create_address(input, customer_id, address_type_id, &txn).await?;
        address.address_id = Set(address_id);

        let update_address = AddressesEntity::update(address)
            .filter(addresses::Column::AddressId.eq(address_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(update_address.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn delete_address(
        &self,
        ctx: &Context<'_>,
        address_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{
            addresses,
            prelude::{AddressTypes as AddressTypesEntity, Addresses as AddressesEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let address = AddressesEntity::find()
            .filter(addresses::Column::AddressId.eq(address_id))
            .one(&txn)
            .await?
            .ok_or("Address not found")?;

        if address.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        if address.is_default.unwrap_or(false) {
            return Err("Cannot delete default address".into());
        }

        let address_type = AddressTypesEntity::find_by_id(address.address_type_id.unwrap())
            .one(&txn)
            .await?
            .ok_or("Address type not found")?;

        address.delete(&txn).await?;
        address_type.delete(&txn).await?;

        txn.commit().await?;

        Ok("Address deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_address_type(
        &self,
        ctx: &Context<'_>,
        address_type_id: i32,
        name: String,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{
            address_types, addresses,
            prelude::{AddressTypes as AddressTypesEntity, Addresses as AddressesEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let address_type = AddressTypesEntity::find()
            .filter(address_types::Column::AddressTypeId.eq(address_type_id))
            .one(&txn)
            .await?
            .ok_or("Address type not found")?;

        let address = AddressesEntity::find()
            .filter(addresses::Column::AddressTypeId.eq(address_type_id))
            .one(&txn)
            .await?
            .ok_or("Address not found")?;

        if address.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        let mut address_type: address_types::ActiveModel = address_type.into();
        address_type.name = Set(name);

        address_type.update(&txn).await?;

        txn.commit().await?;

        Ok("Address type updated".to_string())
    }
}
