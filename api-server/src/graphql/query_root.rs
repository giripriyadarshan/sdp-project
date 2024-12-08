use crate::models::products::Reviews;
use crate::{
    auth::auth::{RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        products::{Categories, Products},
        user::{Customers, Suppliers, Users},
    },
};
use async_graphql::{Context, Object};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;

pub struct QueryRoot;

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
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
        use sea_orm::{DatabaseConnection, EntityTrait};
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

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn get_user(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Users, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::users;
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
        let db = ctx.data::<DatabaseConnection>()?;

        let user = users::Entity::find()
            .filter(users::Column::UserId.eq(Auth::verify_token(&token)?.user_id))
            .one(db)
            .await
            .map_err(|_| "User not found")?
            .map(|user| user.into())
            .unwrap();

        Ok(user)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn customer_profile(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Customers, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::customers;
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
        let db = ctx.data::<DatabaseConnection>()?;

        let customer = customers::Entity::find()
            .filter(customers::Column::UserId.eq(Auth::verify_token(&token)?.user_id))
            .one(db)
            .await
            .map_err(|_| "Customer not found")?
            .map(|customer| customer.into())
            .unwrap();

        Ok(customer)
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn supplier_profile(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Suppliers, async_graphql::Error> {
        use crate::auth::auth::Auth;
        use crate::entity::suppliers;
        use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
        let db = ctx.data::<DatabaseConnection>()?;

        let supplier = suppliers::Entity::find()
            .filter(suppliers::Column::UserId.eq(Auth::verify_token(&token)?.user_id))
            .one(db)
            .await
            .map_err(|_| "Supplier not found")?
            .map(|supplier| supplier.into())
            .unwrap();

        Ok(supplier)
    }

    async fn reviews_for_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
    ) -> Result<Vec<Reviews>, async_graphql::Error> {
        use crate::entity::reviews;
        use sea_orm::{DatabaseConnection, EntityTrait};
        let db = ctx.data::<DatabaseConnection>()?;

        let reviews = reviews::Entity::find()
            .filter(reviews::Column::ProductId.eq(product_id))
            .all(db)
            .await?;

        let reviews: Vec<Reviews> = reviews.into_iter().map(|review| review.into()).collect();

        Ok(reviews)
    }
}
