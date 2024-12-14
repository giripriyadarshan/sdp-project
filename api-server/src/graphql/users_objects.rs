use crate::{
    auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::user::{
        Customers, LoginUser, RegisterCustomer, RegisterSupplier, RegisterUser, Suppliers, Users,
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    ActiveEnum, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
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
            .filter(suppliers::Column::UserId.eq(Auth::verify_token(token)?.user_id))
            .one(db)
            .await
            .map_err(|_| "Supplier not found")?
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
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{prelude::Users as UsersEntity, users};

        let db = ctx.data::<DatabaseConnection>()?;

        let user: Users = UsersEntity::find()
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

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn refresh_token(&self, ctx: &Context<'_>) -> Result<String, async_graphql::Error> {
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        Ok(Auth::refresh_token(token)?)
    }
}
