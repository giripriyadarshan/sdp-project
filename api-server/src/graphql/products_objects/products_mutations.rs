use crate::{
    auth::{RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        products::{
            check_if_supplier_owns_product, create_discount_model, create_product_model,
            create_review_model, Discounts, Products, RegisterDiscount, RegisterProduct,
            RegisterReview, Reviews,
        },
        user::get_customer_supplier_id,
    },
};
use async_graphql::{Context, Object};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, TransactionTrait,
};

#[derive(Default)]
pub struct ProductsMutation;

#[Object]
impl ProductsMutation {
    // needs a role_guard to check if the user is a supplier as even a customer has a valid token
    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_product(
        &self,
        ctx: &Context<'_>,
        input: RegisterProduct,
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::prelude::Products as ProductsEntity;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;
        let product = create_product_model(input, supplier_id)?;
        let insert_product = ProductsEntity::insert(product)
            .exec_with_returning(db)
            .await?;
        Ok(insert_product.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn update_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        input: RegisterProduct,
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::{prelude::Products as ProductsEntity, products};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;
        check_if_supplier_owns_product(db, supplier_id, product_id).await?;
        let mut product = create_product_model(input, supplier_id)?;

        product.product_id = Set(product_id);
        let update_product = ProductsEntity::update(product)
            .filter(products::Column::ProductId.eq(product_id))
            .exec(db)
            .await?;
        Ok(update_product.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn delete_product(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::prelude::Products as ProductsEntity;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, product_id).await?;
        ProductsEntity::delete_by_id(product_id).exec(db).await?;
        Ok("Product deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_review(
        &self,
        ctx: &Context<'_>,
        input: RegisterReview,
    ) -> Result<Reviews, async_graphql::Error> {
        use crate::entity::{
            orders,
            prelude::{
                OrderItems as OrderItemsEntity, Orders as OrdersEntity, Reviews as ReviewsEntity,
            },
        };
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        // check if customer has ordered the product
        if OrderItemsEntity::find()
            .inner_join(OrdersEntity)
            .filter(orders::Column::CustomerId.eq(customer_id))
            .one(&txn)
            .await?
            .is_none()
        {
            return Err("Customer has not ordered the product".into());
        }

        let review = create_review_model(input, customer_id)?;

        let insert_review = ReviewsEntity::insert(review)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(insert_review.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_review(
        &self,
        ctx: &Context<'_>,
        review_id: i32,
        input: RegisterReview,
    ) -> Result<Reviews, async_graphql::Error> {
        use crate::entity::{prelude::Reviews as ReviewsEntity, reviews};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let review = create_review_model(input, customer_id)?;

        let update_review = ReviewsEntity::update(review)
            .filter(reviews::Column::ReviewId.eq(review_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(update_review.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn delete_review(
        &self,
        ctx: &Context<'_>,
        review_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{prelude::Reviews as ReviewsEntity, reviews};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;

        let review = ReviewsEntity::find()
            .filter(reviews::Column::ReviewId.eq(review_id))
            .one(&txn)
            .await?
            .ok_or("Review not found")?;

        if review.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        review.delete(&txn).await?;

        txn.commit().await?;

        Ok("Review deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_discount(
        &self,
        ctx: &Context<'_>,
        input: RegisterDiscount,
    ) -> Result<Discounts, async_graphql::Error> {
        use crate::entity::prelude::Discounts as DiscountsEntity;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, input.product_id).await?;

        let discount = create_discount_model(input)?;
        let insert_discount = DiscountsEntity::insert(discount)
            .exec_with_returning(db)
            .await?;
        Ok(insert_discount.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn update_discount(
        &self,
        ctx: &Context<'_>,
        discount_id: i32,
        input: RegisterDiscount,
    ) -> Result<Discounts, async_graphql::Error> {
        use crate::entity::{discounts, prelude::Discounts as DiscountsEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, input.product_id).await?;

        let mut discount = create_discount_model(input)?;
        discount.discount_id = Set(discount_id);

        let update_discount = DiscountsEntity::update(discount)
            .filter(discounts::Column::DiscountId.eq(discount_id))
            .exec(db)
            .await?;
        Ok(update_discount.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn delete_discount(
        &self,
        ctx: &Context<'_>,
        discount_id: i32,
        product_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::prelude::Discounts as DiscountsEntity;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, product_id).await?;

        DiscountsEntity::delete_by_id(discount_id).exec(db).await?;
        Ok("Discount deleted".to_string())
    }
}
