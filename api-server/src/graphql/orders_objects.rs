use crate::{
    auth::{RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        bills::Bills,
        orders::{Orders, RegisterOrder},
        products::Products,
        user::get_customer_supplier_id,
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    prelude::Decimal, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, TransactionTrait,
};

#[derive(Default)]
pub struct OrdersQuery;

#[derive(Default)]
pub struct OrdersMutation;

#[Object]
impl OrdersQuery {
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
}

#[Object]
impl OrdersMutation {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_order(
        &self,
        ctx: &Context<'_>,
        input: RegisterOrder,
    ) -> Result<Orders, async_graphql::Error> {
        use crate::entity::{
            discounts, order_items, orders,
            prelude::{
                Discounts as DiscountsEntity, OrderItems as OrderItemsEntity,
                Orders as OrdersEntity, Products as ProductsEntity,
            },
            products,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let discount_id = match &input.discount_code {
            Some(discount_code) => ProductsEntity::find()
                .filter(products::Column::Name.eq(discount_code))
                .one(db)
                .await
                .map_err(|_| "Discount not found")?
                .map(|product| product.product_id),
            None => None,
        };

        let mut total_amount: f64 = 0.0;
        for item in &input.order_items {
            let product: products::Model = ProductsEntity::find_by_id(item.product_id)
                .one(db)
                .await
                .map_err(|_| "Product not found")?
                .unwrap();
            total_amount += product.base_price.to_string().parse::<f64>()? * item.quantity as f64;
        }

        if let Some(discount_id) = discount_id {
            let discount: discounts::Model = DiscountsEntity::find_by_id(discount_id)
                .one(&txn)
                .await?
                .unwrap();

            if discount.discount_type == "PERCENTAGE" {
                total_amount -=
                    total_amount * discount.discount_value.to_string().parse::<f64>()? / 100.0;
            } else {
                total_amount -= discount.discount_value.to_string().parse::<f64>()?;
            }

            // increment discount usage
            let mut discount: discounts::ActiveModel = discount.into();
            discount.times_used = Set(Some(discount.times_used.unwrap().unwrap() + 1));
        }

        let order = orders::ActiveModel {
            customer_id: Set(customer_id),
            shipping_address_id: Set(input.shipping_address_id),
            payment_method_id: Set(input.payment_method_id),
            discount_id: Set(discount_id),
            total_amount: Set(Decimal::from_str_exact(total_amount.to_string().as_str())?),
            status: Set("PENDING".to_string()),
            ..Default::default()
        };

        let insert_order = OrdersEntity::insert(order)
            .exec_with_returning(&txn)
            .await?;

        for item in &input.order_items {
            let product: products::Model = ProductsEntity::find_by_id(item.product_id)
                .one(&txn)
                .await?
                .unwrap();

            if product.stock_quantity < item.quantity {
                return Err("Insufficient stock".into());
            }

            let product_base_price = product.base_price;

            let product: products::ActiveModel = products::ActiveModel {
                stock_quantity: Set(product.stock_quantity - item.quantity),
                ..product.into()
            };

            ProductsEntity::update(product)
                .filter(products::Column::ProductId.eq(item.product_id))
                .exec(&txn)
                .await?;

            let order_item = order_items::ActiveModel {
                order_id: Set(insert_order.order_id),
                product_id: Set(item.product_id),
                quantity: Set(item.quantity),
                unit_price: Set(product_base_price),
                ..Default::default()
            };
            OrderItemsEntity::insert(order_item).exec(&txn).await?;
        }

        txn.commit().await?;

        Ok(insert_order.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn update_order_status(
        &self,
        ctx: &Context<'_>,
        order_id: i32,
        status: String,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{orders, prelude::Orders as OrdersEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let txn = db.begin().await?;

        let order: orders::Model = OrdersEntity::find_by_id(order_id)
            .one(&txn)
            .await
            .map_err(|_| "Order not found")?
            .unwrap();

        let mut update_order: orders::ActiveModel = order.into();
        update_order.status = Set(status);

        update_order.update(&txn).await?;

        txn.commit().await?;

        Ok("Order status updated".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn cancel_order(
        &self,
        ctx: &Context<'_>,
        order_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{
            order_items, orders,
            prelude::{Orders as OrdersEntity, Products as ProductsEntity},
            products,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let order: orders::Model = OrdersEntity::find_by_id(order_id)
            .one(&txn)
            .await
            .map_err(|_| "Order not found")?
            .unwrap();

        if order.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        if order.status == "CANCELLED" {
            return Err("Order already cancelled".into());
        }

        let order_items_list = order_items::Entity::find()
            .filter(order_items::Column::OrderId.eq(order_id))
            .all(&txn)
            .await?;

        for order_item in order_items_list {
            let product: products::Model = ProductsEntity::find_by_id(order_item.product_id)
                .one(&txn)
                .await?
                .unwrap();

            let product: products::ActiveModel = products::ActiveModel {
                stock_quantity: Set(product.stock_quantity + order_item.quantity),
                ..product.into()
            };

            ProductsEntity::update(product)
                .filter(products::Column::ProductId.eq(order_item.product_id))
                .exec(&txn)
                .await?;
        }

        let mut order: orders::ActiveModel = order.into();

        order.status = Set("CANCELLED".to_string());

        order.update(&txn).await?;

        txn.commit().await?;

        Ok("Order cancelled".to_string())
    }
}
