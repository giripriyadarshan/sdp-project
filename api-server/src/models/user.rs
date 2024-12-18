use crate::{
    auth::Auth,
    entity::{
        customers::Model as CustomersModel, suppliers::Model as SuppliersModel,
        users::Model as UsersModel,
    },
};
use async_graphql::{Error, InputObject, SimpleObject};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveEnum, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};

#[derive(SimpleObject)]
pub struct Users {
    pub user_id: i32,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub email_verified: Option<bool>,
}

impl From<UsersModel> for Users {
    fn from(val: UsersModel) -> Users {
        Users {
            user_id: val.user_id,
            email: val.email,
            password: val.password,
            role: val.role.to_value(),
            created_at: val.created_at,
            email_verified: val.email_verified,
        }
    }
}

#[derive(InputObject)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(SimpleObject)]
pub struct AuthUser {
    pub token: String,
    pub user_role: String,
}

#[derive(InputObject)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(SimpleObject)]
pub struct Customers {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub registration_date: Option<DateTimeWithTimeZone>,
    pub user_id: i32,
}

impl From<CustomersModel> for Customers {
    fn from(val: CustomersModel) -> Customers {
        Customers {
            customer_id: val.customer_id,
            first_name: val.first_name,
            last_name: val.last_name,
            registration_date: val.registration_date,
            user_id: val.user_id,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterCustomer {
    pub first_name: String,
    pub last_name: String,
}

#[derive(SimpleObject)]
pub struct Suppliers {
    pub supplier_id: i32,
    pub name: String,
    pub contact_phone: Option<String>,
    pub user_id: i32,
}

impl From<SuppliersModel> for Suppliers {
    fn from(val: SuppliersModel) -> Suppliers {
        Suppliers {
            supplier_id: val.supplier_id,
            name: val.name,
            contact_phone: val.contact_phone,
            user_id: val.user_id,
        }
    }
}

#[derive(InputObject)]
pub struct RegisterSupplier {
    pub name: String,
    pub contact_phone: Option<String>,
}

pub async fn get_customer_supplier_id(
    db: &DatabaseConnection,
    token: &str,
    role: &str,
) -> Result<i32, Error> {
    use crate::entity::{customers, suppliers};

    match role {
        "supplier" => {
            let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
            suppliers::Entity::find()
                .filter(suppliers::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .map(|supplier| supplier.supplier_id)
                .ok_or_else(|| Error::new("Supplier not found"))
        }

        "customer" => {
            let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
            customers::Entity::find()
                .filter(customers::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .map(|customer| customer.customer_id)
                .ok_or_else(|| Error::new("Customer not found"))
        }
        _ => Err(Error::new("Invalid role")),
    }
}
