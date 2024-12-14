use crate::{
    auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        addresses::Addresses,
        bills::Bills,
        order_und_pagination::{OrderAndPagination, PageInfo},
        orders::Orders,
        payments::{CardTypes, PaymentMethods},
        products::{
            paginate_products, Categories, Discounts, Products, ProductsPaginate, Reviews,
            ReviewsPaginate,
        },
        user::{get_customer_supplier_id, Customers, Suppliers, Users},
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
    ) -> Result<ProductsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Products as ProductsEntity, products};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products =
            ProductsEntity::find().filter(match (category_id, supplier_id, base_product_id) {
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
        let products = products.paginate(db, page_size);
        let items = PageInfo {
            total_pages: products.num_pages().await?,
            total_items: products.num_items().await?,
        };

        let products = products.fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(ProductsPaginate {
            products,
            page_info: items,
        })
    }

    async fn products_with_name(
        &self,
        ctx: &Context<'_>,
        name: String,
        paginator: OrderAndPagination,
    ) -> Result<ProductsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Products as ProductsEntity, products};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let products = ProductsEntity::find().filter(products::Column::Name.contains(name));

        let products = paginate_products(paginator, products).await?;

        let products = products.paginate(db, page_size);
        let items = PageInfo {
            total_pages: products.num_pages().await?,
            total_items: products.num_items().await?,
        };

        let products = products.fetch_page(page).await?;

        let products: Vec<Products> = products.into_iter().map(|product| product.into()).collect();

        Ok(ProductsPaginate {
            products,
            page_info: items,
        })
    }

    async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<Categories>, async_graphql::Error> {
        use crate::entity::prelude::Categories as CategoriesEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let categories = CategoriesEntity::find().all(db).await?;

        let categories: Vec<Categories> = categories
            .into_iter()
            .map(|category| category.into())
            .collect();

        Ok(categories)
    }

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

        let customer = CustomersEntity::find()
            .filter(customers::Column::UserId.eq(Auth::verify_token(token)?.user_id))
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

    async fn reviews_for_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        paginator: OrderAndPagination,
    ) -> Result<ReviewsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Reviews as ReviewsEntity, reviews};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let reviews = ReviewsEntity::find().filter(reviews::Column::ProductId.eq(product_id));

        let reviews = reviews
            .order_by_asc(reviews::Column::ReviewDate)
            .paginate(db, page_size);

        let items = PageInfo {
            total_pages: reviews.num_pages().await?,
            total_items: reviews.num_items().await?,
        };

        let reviews = reviews.fetch_page(page).await?;

        let reviews: Vec<Reviews> = reviews.into_iter().map(|review| review.into()).collect();

        Ok(ReviewsPaginate {
            reviews,
            page_info: items,
        })
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn addresses(&self, ctx: &Context<'_>) -> Result<Vec<Addresses>, async_graphql::Error> {
        use crate::entity::addresses;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
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
        use crate::entity::{
            customers,
            prelude::{Customers as CustomersEntity, PaymentMethods as PaymentMethodsEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let user_id = Auth::verify_token(token)?.user_id.parse::<i32>()?;
        let payment_method = PaymentMethodsEntity::find()
            .inner_join(CustomersEntity)
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
        use crate::entity::prelude::CardTypes as CardTypesEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let card_type = CardTypesEntity::find_by_id(card_type_id).one(db).await?;

        Ok(card_type.unwrap().into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn orders(&self, ctx: &Context<'_>) -> Result<Vec<Orders>, async_graphql::Error> {
        use crate::entity::{orders, prelude::Orders as OrdersEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let orders = OrdersEntity::find()
            .filter(orders::Column::CustomerId.eq(customer_id))
            .all(db)
            .await?;

        let orders: Vec<Orders> = orders.into_iter().map(|order| order.into()).collect();

        Ok(orders)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn order_items(
        &self,
        ctx: &Context<'_>,
        order_id: i32,
    ) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::{
            order_items,
            prelude::{OrderItems as OrdersItemsEntity, Products as ProductsEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;

        let order_items = OrdersItemsEntity::find()
            .filter(order_items::Column::OrderId.eq(order_id))
            .all(db)
            .await?;

        let mut products_list = Vec::new();

        for order_item in &order_items {
            products_list.push(
                ProductsEntity::find_by_id(order_item.product_id)
                    .one(db)
                    .await?
                    .unwrap()
                    .into(),
            );
        }

        Ok(products_list)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn cart_items(&self, ctx: &Context<'_>) -> Result<Vec<Products>, async_graphql::Error> {
        use crate::entity::{
            cart_items,
            prelude::{
                CartItems as CartItemsEntity, Products as ProductsEntity,
                ShoppingCarts as ShoppingCartsEntity,
            },
            shopping_carts,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let cart_id = ShoppingCartsEntity::find()
            .filter(shopping_carts::Column::CustomerId.eq(customer_id))
            .one(db)
            .await?
            .unwrap()
            .cart_id;

        let cart_items = CartItemsEntity::find()
            .filter(cart_items::Column::CartId.eq(cart_id))
            .all(db)
            .await?;

        let mut products_list = Vec::new();

        for cart_item in &cart_items {
            products_list.push(
                ProductsEntity::find_by_id(cart_item.product_id)
                    .one(db)
                    .await?
                    .unwrap()
                    .into(),
            );
        }

        Ok(products_list)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn bills(&self, ctx: &Context<'_>) -> Result<Vec<Bills>, async_graphql::Error> {
        use crate::entity::{
            bills, orders, prelude::Bills as BillsEntity, prelude::Orders as OrdersEntity,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let orders = OrdersEntity::find()
            .filter(orders::Column::CustomerId.eq(customer_id))
            .all(db)
            .await?;

        let mut bills_list = Vec::new();

        for order in &orders {
            let bill = BillsEntity::find()
                .filter(bills::Column::OrderId.eq(order.order_id))
                .one(db)
                .await?;
            bills_list.push(bill.unwrap().into());
        }

        Ok(bills_list)
    }

    async fn reviews(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        paginator: OrderAndPagination,
    ) -> Result<ReviewsPaginate, async_graphql::Error> {
        use crate::entity::{prelude::Reviews as ReviewsEntity, reviews};
        let db = ctx.data::<DatabaseConnection>()?;

        let page = paginator.pagination.page - 1;
        let page_size = paginator.pagination.page_size;

        let reviews = ReviewsEntity::find().filter(reviews::Column::ProductId.eq(product_id));

        let reviews = reviews
            .order_by_asc(reviews::Column::ReviewDate)
            .paginate(db, page_size);

        let items = PageInfo {
            total_pages: reviews.num_pages().await?,
            total_items: reviews.num_items().await?,
        };

        let reviews = reviews.fetch_page(page).await?;

        let reviews: Vec<Reviews> = reviews.into_iter().map(|review| review.into()).collect();

        Ok(ReviewsPaginate {
            reviews,
            page_info: items,
        })
    }

    async fn discounts(&self, ctx: &Context<'_>) -> Result<Vec<Discounts>, async_graphql::Error> {
        use crate::entity::prelude::Discounts as DiscountsEntity;
        let db = ctx.data::<DatabaseConnection>()?;

        let discounts = DiscountsEntity::find().all(db).await?;

        let discounts: Vec<Discounts> = discounts
            .into_iter()
            .map(|discount| discount.into())
            .collect();

        Ok(discounts)
    }

    async fn discounts_on_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
    ) -> Result<Vec<Discounts>, async_graphql::Error> {
        use crate::entity::{discounts, prelude::Discounts as DiscountsEntity};
        let db = ctx.data::<DatabaseConnection>()?;

        let discounts = DiscountsEntity::find()
            .filter(discounts::Column::ProductId.eq(product_id))
            .all(db)
            .await?;

        let discounts: Vec<Discounts> = discounts
            .into_iter()
            .map(|discount| discount.into())
            .collect();

        Ok(discounts)
    }
}
