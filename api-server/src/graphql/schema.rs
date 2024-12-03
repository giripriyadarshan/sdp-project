use crate::auth::auth::{RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER};
use crate::models::orders::{Orders, RegisterOrder};
use crate::models::products::Categories;
use crate::models::user::{LoginUser, RegisterCustomer};
use crate::models::{products::Products, user::Customers};
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, EmptySubscription, Object, Schema};
use axum::response;
use axum::response::IntoResponse;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

macro_rules! role_guard {
    ($($role:expr),*) => {
        RoleGuard::new(vec![$($role),*])
    };
}

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn products_with_id(
        &self,
        ctx: &Context<'_>,
        category_id: Option<i32>,
        supplier_id: Option<i32>,
        base_product_id: Option<i32>,
    ) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;

        let products = products::Entity::find()
            .filter(match (category_id, supplier_id, base_product_id) {
                (Some(category_id), None, None) => products::Column::CategoryId.eq(category_id),
                (None, Some(supplier_id), None) => products::Column::SupplierId.eq(supplier_id),
                (None, None, Some(base_product_id)) => {
                    products::Column::BaseProductId.eq(base_product_id)
                }
                _ => products::Column::CategoryId
                    .eq(category_id)
                    .and(products::Column::SupplierId.eq(supplier_id))
                    .and(products::Column::BaseProductId.eq(base_product_id)),
            })
            .all(db)
            .await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(products)
    }

    async fn products_with_name(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;

        let products = products::Entity::find()
            .filter(products::Column::Name.contains(name))
            .all(db)
            .await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(products)
    }

    async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<Categories>, async_graphql::Error> {
        use crate::entity::categories;
        let db = ctx.data::<DatabaseConnection>()?;

        let categories = categories::Entity::find().all(db).await?;

        let categories: Vec<Categories> = categories
            .into_iter()
            .map(|category| Categories {
                category_id: category.category_id,
                name: category.name,
                parent_category_id: category.parent_category_id,
            })
            .collect();

        Ok(categories)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
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
