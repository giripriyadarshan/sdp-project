use crate::models::user::AuthUser;
use crate::{
    auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::user::{
        Customers, LoginUser, RegisterCustomer, RegisterSupplier, RegisterUser, Suppliers, Users,
    },
};
use async_graphql::{Context, Object};
use chrono::Duration;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};

#[derive(Default)]
pub struct UsersQuery;

#[derive(Default)]
pub struct UsersMutation;

#[Object]
impl UsersQuery {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn get_user(&self, ctx: &Context<'_>) -> Result<Users, async_graphql::Error> {
        use crate::entity::prelude::Users as UsersEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user =
            UsersEntity::find_by_id(Auth::verify_token(token)?.user_id.parse::<i32>().unwrap())
                .one(db)
                .await
                .map_err(|_| "User not found")?
                .map(|user| user.into())
                .unwrap();

        Ok(user)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn customer_profile(&self, ctx: &Context<'_>) -> Result<Customers, async_graphql::Error> {
        use crate::entity::{customers, prelude::Customers as CustomersEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;

        let customer = CustomersEntity::find()
            .filter(customers::Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(|_| "Customer not found")?
            .map(|customer| customer.into())
            .unwrap();

        Ok(customer)
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn supplier_profile(&self, ctx: &Context<'_>) -> Result<Suppliers, async_graphql::Error> {
        use crate::entity::{prelude::Suppliers as SuppliersEntity, suppliers};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let supplier = SuppliersEntity::find()
            .filter(
                suppliers::Column::UserId
                    .eq(Auth::verify_token(token)?.user_id.parse::<i32>().unwrap()),
            )
            .one(db)
            .await
            .map_err(|e| format!("{}", e))?
            .map(|supplier| supplier.into())
            .unwrap();

        Ok(supplier)
    }
}

#[Object]
impl UsersMutation {
    async fn register_user(
        &self,
        ctx: &Context<'_>,
        input: RegisterUser,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{prelude::Users as UsersEntity, sea_orm_active_enums::UserRole, users};

        Auth::check_email(&input.email)?;

        if UsersEntity::find()
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
        let insert_user = UsersEntity::insert(user).exec_with_returning(db).await?;

        Ok(Auth::create_token(
            insert_user.user_id,
            insert_user.role.to_value(),
            Duration::days(30),
        )?)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_customer(
        &self,
        ctx: &Context<'_>,
        input: RegisterCustomer,
    ) -> Result<Customers, async_graphql::Error> {
        use crate::entity::{customers, prelude::Customers as CustomersEntity};
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let db = ctx.data::<DatabaseConnection>()?;

        let customer = customers::ActiveModel {
            first_name: Set(input.first_name),
            last_name: Set(input.last_name),
            user_id: Set(Auth::verify_token(token)?.user_id.parse::<i32>()?),
            ..Default::default()
        };

        let insert_customer = CustomersEntity::insert(customer)
            .exec_with_returning(db)
            .await?;

        Ok(insert_customer.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_supplier(
        &self,
        ctx: &Context<'_>,
        input: RegisterSupplier,
    ) -> Result<Suppliers, async_graphql::Error> {
        use crate::entity::{prelude::Suppliers as SuppliersEntity, suppliers};

        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let supplier = suppliers::ActiveModel {
            user_id: Set(Auth::verify_token(token)?.user_id.parse::<i32>()?),
            name: Set(input.name),
            contact_phone: Set(input.contact_phone),
            ..Default::default()
        };

        let insert_supplier = SuppliersEntity::insert(supplier)
            .exec_with_returning(db)
            .await?;

        Ok(insert_supplier.into())
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        login_details: LoginUser,
    ) -> Result<AuthUser, async_graphql::Error> {
        use crate::entity::{prelude::Users as UsersEntity, users};

        let db = ctx.data::<DatabaseConnection>()?;

        let user: Users = UsersEntity::find()
            .filter(users::Column::Email.eq(&login_details.email))
            .one(db)
            .await
            .map_err(|_| "User not found")?
            .map(|user| user.into())
            .unwrap();

        let token = match Auth::verify_password(&login_details.password, &user.password) {
            Ok(verification_status) => {
                if verification_status {
                    Auth::create_token(user.user_id, user.role.clone(), Duration::days(30))?
                } else {
                    return Err("Invalid password".into());
                }
            }
            Err(_) => return Err("Password not readable, please reset password".into()),
        };

        Ok(AuthUser {
            user_role: user.role,
            token,
        })
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn refresh_token(&self, ctx: &Context<'_>) -> Result<String, async_graphql::Error> {
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        Ok(Auth::refresh_token(token)?)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn change_password(
        &self,
        ctx: &Context<'_>,
        old_password: String,
        new_password: String,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{prelude::Users as UsersEntity, users};

        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
        let db = ctx.data::<DatabaseConnection>()?;

        let user = UsersEntity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|_| "User not found")?
            .unwrap();

        let user_model: Users = user.clone().into();

        match Auth::verify_password(&old_password, &user_model.password) {
            Ok(verification_status) => {
                if verification_status {
                    let new_password = Auth::hash_password(&new_password)?;
                    let mut user: users::ActiveModel = user.into();
                    user.password = Set(new_password);
                    user.update(db).await?;
                    Ok("Password updated successfully".to_string())
                } else {
                    Err("Invalid password".into())
                }
            }
            Err(_) => Err("Password not readable, please reset password".into()),
        }
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn send_email_verification(
        &self,
        ctx: &Context<'_>,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::prelude::Users as UsersEntity;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
        let db = ctx.data::<DatabaseConnection>()?;

        let user = UsersEntity::find_by_id(user_id).one(db).await?.unwrap();
        let user_email = user.email;
        let user_id = user.user_id;
        let user_role = user.role.to_value();

        Ok(Auth::send_email_verification(user_email, user_id, user_role).await?)
    }
}
