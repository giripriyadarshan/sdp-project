use crate::models::products::paginate_products;
use crate::{
    auth::auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        addresses::Addresses,
        order_und_pagination::OrderAndPagination,
        payments::{CardTypes, PaymentMethods},
        products::{Categories, Products, Reviews},
        user::{Customers, Suppliers, Users},
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Statement,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn products_with_id(
        &self,
        ctx: &Context<'_>,
        category_id: Option<i32>,
        supplier_id: Option<i32>,
        base_product_id: Option<i32>,
        paginator: OrderAndPagination,
    ) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products =
            products::Entity::find().filter(match (category_id, supplier_id, base_product_id) {
                (Some(category_id), None, None) => products::Column::CategoryId.eq(category_id),
                (None, Some(supplier_id), None) => products::Column::SupplierId.eq(supplier_id),
                (None, None, Some(base_product_id)) => {
                    products::Column::BaseProductId.eq(base_product_id)
                }
                _ => products::Column::CategoryId
                    .eq(category_id)
                    .and(products::Column::SupplierId.eq(supplier_id))
                    .and(products::Column::BaseProductId.eq(base_product_id)),
            });

        let products = paginate_products(paginator, products).await?;

        let products = products.paginate(db, page_size).fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(products)
    }

    async fn products_with_name(
        &self,
        ctx: &Context<'_>,
        name: String,
        paginator: OrderAndPagination,
    ) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products = products::Entity::find().filter(products::Column::Name.contains(name));

        let products = paginate_products(paginator, products).await?;

        let products = products.paginate(db, page_size).fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(products)
    }

    async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<Categories>, async_graphql::Error> {
        use crate::entity::categories;
        let db = ctx.data::<DatabaseConnection>()?;

        let categories = categories::Entity::find().all(db).await?;

        let categories: Vec<Categories> = categories
            .into_iter()
            .map(|category| category.into())
            .collect();

        Ok(categories)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER, ROLE_SUPPLIER)")]
    async fn get_user(&self, ctx: &Context<'_>) -> Result<Users, async_graphql::Error> {
        use crate::entity::users;
        let db = ctx.data::<DatabaseConnection>()?;

        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user =
            users::Entity::find_by_id(Auth::verify_token(&token)?.user_id.parse::<i32>().unwrap())
                .one(db)
                .await
                .map_err(|_| "User not found")?
                .map(|user| user.into())
                .unwrap();

        Ok(user)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn customer_profile(&self, ctx: &Context<'_>) -> Result<Customers, async_graphql::Error> {
        use crate::entity::customers;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

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
    async fn supplier_profile(&self, ctx: &Context<'_>) -> Result<Suppliers, async_graphql::Error> {
        use crate::entity::suppliers;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

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
        paginator: OrderAndPagination,
    ) -> Result<Vec<Reviews>, async_graphql::Error> {
        use crate::entity::reviews;
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let reviews = reviews::Entity::find().filter(reviews::Column::ProductId.eq(product_id));

        let reviews = reviews
            .order_by_asc(reviews::Column::ReviewDate)
            .paginate(db, page_size)
            .fetch_page(page)
            .await?;

        let reviews: Vec<Reviews> = reviews.into_iter().map(|review| review.into()).collect();

        Ok(reviews)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn addresses(&self, ctx: &Context<'_>) -> Result<Vec<Addresses>, async_graphql::Error> {
        use crate::entity::addresses;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(&token)?.user_id.parse::<i32>()?;
        // this query includes inner join with users, customers and addresses tables
        // this query can be written entirely with SELECT and WHERE. Basically, get customer_id using user_id in customer table and then insert it into addresses table. But this has the keyword "JOIN" in it, so we'll go with this one.
        // For reference: SELECT * FROM addresses WHERE customer_id = (SELECT customer_id FROM customers WHERE user_id = $1);
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
    async fn payment_methods(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<PaymentMethods>, async_graphql::Error> {
        use crate::entity::{customers, payment_methods};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(&token)?.user_id.parse::<i32>()?;
        let payment_method = payment_methods::Entity::find()
            .inner_join(customers::Entity)
            .filter(customers::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let payment_methods: Vec<PaymentMethods> = payment_method
            .into_iter()
            .map(|payment_methods| payment_methods.into())
            .collect();

        Ok(payment_methods)
    }

    async fn card_type(
        &self,
        ctx: &Context<'_>,
        card_type_id: i32,
    ) -> Result<CardTypes, async_graphql::Error> {
        use crate::entity::card_types;
        let db = ctx.data::<DatabaseConnection>()?;

        let card_type = card_types::Entity::find_by_id(card_type_id).one(db).await?;

        Ok(card_type.unwrap().into())
    }
}
