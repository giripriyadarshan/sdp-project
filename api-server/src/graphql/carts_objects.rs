use crate::{
    auth::{RoleGuard, ROLE_CUSTOMER},
    graphql::macros::role_guard,
    models::{
        products::{check_product_exists, Products},
        user::get_customer_supplier_id,
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, TransactionTrait,
};

#[derive(Default)]
pub struct CartsQuery;

#[derive(Default)]
pub struct CartsMutation;

#[Object]
impl CartsQuery {
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
}

#[Object]
impl CartsMutation {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn add_to_cart(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        quantity: i32,
    ) -> Result<i32, async_graphql::Error> {
        use crate::entity::{
            cart_items,
            prelude::{CartItems as CartItemsEntity, ShoppingCarts as ShoppingCartsEntity},
            shopping_carts,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = match ShoppingCartsEntity::find()
            .filter(shopping_carts::Column::CustomerId.eq(customer_id))
            .one(&txn)
            .await?
        {
            Some(cart) => cart,
            None => {
                let new_cart = shopping_carts::ActiveModel {
                    customer_id: Set(customer_id),
                    ..Default::default()
                };
                new_cart.insert(&txn).await?
            }
        };

        let cart_item = cart_items::ActiveModel {
            cart_id: Set(cart.cart_id),
            product_id: Set(product_id),
            quantity: Set(quantity),
            ..Default::default()
        };

        CartItemsEntity::insert(cart_item).exec(&txn).await?;
        txn.commit().await?;

        Ok(cart.cart_id)
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_cart_item_quantity(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        quantity: i32,
        cart_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{
            cart_items,
            prelude::{CartItems as CartItemsEntity, ShoppingCarts as ShoppingCartsEntity},
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = ShoppingCartsEntity::find_by_id(cart_id)
            .one(&txn)
            .await?
            .ok_or("Cart not found")?;

        let cart_item = CartItemsEntity::find()
            .filter(cart_items::Column::CartId.eq(cart.cart_id))
            .filter(cart_items::Column::ProductId.eq(product_id))
            .one(&txn)
            .await?
            .ok_or("Product not found in cart")?;

        if cart.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        if quantity == 0 {
            cart_item.delete(&txn).await?;
            txn.commit().await?;
            Ok("Product removed from cart".to_string())
        } else {
            let mut cart_item: cart_items::ActiveModel = cart_item.into();
            cart_item.quantity = Set(quantity);
            cart_item.update(&txn).await?;
            txn.commit().await?;
            Ok("Cart item quantity updated".to_string())
        }
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn remove_from_cart(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{
            cart_items,
            prelude::{CartItems as CartItemsEntity, ShoppingCarts as ShoppingCartsEntity},
            shopping_carts,
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = match ShoppingCartsEntity::find()
            .filter(shopping_carts::Column::CustomerId.eq(customer_id))
            .one(&txn)
            .await?
        {
            Some(cart) => cart,
            None => {
                return Err("Cart not found".into());
            }
        };

        let cart_item = CartItemsEntity::find()
            .filter(cart_items::Column::CartId.eq(cart.cart_id))
            .filter(cart_items::Column::ProductId.eq(product_id))
            .one(&txn)
            .await?
            .ok_or("Product not found in cart")?;

        cart_item.delete(&txn).await?;

        txn.commit().await?;

        Ok("Product removed from cart".to_string())
    }
}
