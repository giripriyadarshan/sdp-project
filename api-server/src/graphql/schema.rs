use crate::models::orders::{Orders, RegisterOrder};
use crate::models::user::{LoginUser, RegisterCustomer};
use crate::models::{products::Products, user::Customers};
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, EmptySubscription, Object, Schema};
use axum::response;
use axum::response::IntoResponse;
use sea_orm::DatabaseConnection;

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn products(
        &self,
        ctx: &Context<'_>,
        category_id: Option<i32>,
    ) -> Result<Products, async_graphql::Error> {
        let db = ctx.data::<DatabaseConnection>()?;
        // Implement product query logic
        unimplemented!()
    }

    async fn customer_profile(&self, ctx: &Context<'_>) -> Result<Customers, async_graphql::Error> {
        // Implement customer profile query logic
        unimplemented!()
    }
}

#[Object]
impl MutationRoot {
    async fn register_customer(
        &self,
        ctx: &Context<'_>,
        input: RegisterCustomer,
    ) -> Result<Customers, async_graphql::Error> {
        // Implement customer registration logic
        unimplemented!()
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        login_details: LoginUser,
    ) -> Result<String, async_graphql::Error> {
        // Implement login logic
        unimplemented!()
    }

    async fn create_order(
        &self,
        ctx: &Context<'_>,
        input: RegisterOrder,
    ) -> Result<Orders, async_graphql::Error> {
        // Implement order creation logic
        unimplemented!()
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(db: DatabaseConnection, redis: redis::Client) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .data(redis)
        .finish()
}

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}
