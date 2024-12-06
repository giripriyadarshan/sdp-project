use crate::{
    auth::auth::{RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::user::{
        Customers, LoginUser, RegisterCustomer, RegisterSupplier, RegisterUser, Suppliers,
    },
};
use async_graphql::{Context, Object};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register_user(
        &self,
        ctx: &Context<'_>,
        input: RegisterUser,
    ) -> Result<String, async_graphql::Error> {
        use crate::auth::auth::{Auth, ROLE_CUSTOMER, ROLE_SUPPLIER};
        use crate::entity::sea_orm_active_enums::UserRole;
        use crate::entity::users;
        use sea_orm::ActiveValue::Set;
        use sea_orm::{ActiveEnum, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

        if users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(ctx.data::<DatabaseConnection>()?)
            .await?
            .is_some()
        {
            return Err("User already exists".into());
        }

        let db = ctx.data::<DatabaseConnection>()?;

        let role = match input.role.as_str() {
            ROLE_CUSTOMER => UserRole::Customer,
            ROLE_SUPPLIER => UserRole::Supplier,
            _ => return Err("Invalid role".into()),
        };

        let password = match Auth::check_password_strength(&input.password) {
            Ok(_) => Auth::hash_password(&input.password)?,
            Err(e) => return Err(e.into()),
        };

        let user = users::ActiveModel {
            email: Set(input.email),
            password: Set(password),
            role: Set(role),
            ..Default::default()
        };
        let insert_user = users::Entity::insert(user).exec_with_returning(db).await?;

        Ok(Auth::create_token(
            insert_user.user_id,
            insert_user.role.to_value(),
        )?)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_customer(
        &self,
        ctx: &Context<'_>,
        input: RegisterCustomer,
        token: String,
    ) -> Result<Customers, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::customers;
        use sea_orm::ActiveValue::Set;
        use sea_orm::{DatabaseConnection, EntityTrait};

        let db = ctx.data::<DatabaseConnection>()?;

        let customer = customers::ActiveModel {
            first_name: Set(input.first_name),
            last_name: Set(input.last_name),
            user_id: Set(Auth::verify_token(&token)?.user_id.parse::<i32>()?),
            ..Default::default()
        };

        let insert_customer = customers::Entity::insert(customer)
            .exec_with_returning(db)
            .await?;

        Ok(insert_customer.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_supplier(
        &self,
        ctx: &Context<'_>,
        input: RegisterSupplier,
        token: String,
    ) -> Result<Suppliers, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::suppliers;
        use sea_orm::ActiveValue::Set;
        use sea_orm::{DatabaseConnection, EntityTrait};

        let db = ctx.data::<DatabaseConnection>()?;

        let supplier = suppliers::ActiveModel {
            user_id: Set(Auth::verify_token(&token)?.user_id.parse::<i32>()?),
            contact_phone: Set(input.contact_phone),
            ..Default::default()
        };

        let insert_supplier = suppliers::Entity::insert(supplier)
            .exec_with_returning(db)
            .await?;

        Ok(insert_supplier.into())
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        login_details: LoginUser,
    ) -> Result<String, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::users;
        use crate::models::user::Users;
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

        let db = ctx.data::<DatabaseConnection>()?;

        let user: Users = users::Entity::find()
            .filter(users::Column::Email.eq(&login_details.email))
            .one(db)
            .await
            .map_err(|_| "User not found")?
            .map(|user| user.into())
            .unwrap();

        match Auth::verify_password(&login_details.password, &user.password) {
            Ok(verification_status) => {
                if verification_status {
                    Ok(Auth::create_token(user.user_id, user.role)?)
                } else {
                    Err("Invalid password".into())
                }
            }
            Err(_) => Err("Password not readable, please reset password".into()),
        }
    }
}
