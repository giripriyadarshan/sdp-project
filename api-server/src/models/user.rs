use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(SimpleObject)]
pub struct Users {
    pub user_id: i64,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(InputObject)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
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

#[derive(InputObject)]
pub struct RegisterSupplier {
    pub name: String,
    pub contact_phone: Option<String>,
}
