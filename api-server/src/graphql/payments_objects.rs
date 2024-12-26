use crate::{
    auth::{Auth, RoleGuard, ROLE_CUSTOMER},
    graphql::macros::role_guard,
    models::{
        payments::{create_payment_method, CardTypes, PaymentMethods, RegisterPaymentMethod},
        user::get_customer_supplier_id,
    },
};
use async_graphql::{Context, Object};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};

#[derive(Default)]
pub struct PaymentsQuery;

#[derive(Default)]
pub struct PaymentsMutation;

#[Object]
impl PaymentsQuery {
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
}

#[Object]
impl PaymentsMutation {
    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_payment_method(
        &self,
        ctx: &Context<'_>,
        input: RegisterPaymentMethod,
    ) -> Result<PaymentMethods, async_graphql::Error> {
        use crate::entity::prelude::PaymentMethods as PaymentMethodsEntity;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;
        let is_default: Option<bool> = Some(input.is_default.unwrap_or(false));

        let payment_method = create_payment_method(customer_id, is_default, input, &txn).await?;

        let insert_payment_method = PaymentMethodsEntity::insert(payment_method)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(insert_payment_method.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_payment_method(
        &self,
        ctx: &Context<'_>,
        payment_method_id: i32,
        input: RegisterPaymentMethod,
    ) -> Result<PaymentMethods, async_graphql::Error> {
        use crate::entity::{payment_methods, prelude::PaymentMethods as PaymentMethodsEntity};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, token, ROLE_CUSTOMER).await?;
        let is_default: Option<bool> = Some(input.is_default.unwrap_or(false));

        let mut payment_method =
            create_payment_method(customer_id, is_default, input, &txn).await?;
        payment_method.payment_method_id = Set(payment_method_id);

        let update_payment_method = PaymentMethodsEntity::update(payment_method)
            .filter(payment_methods::Column::PaymentMethodId.eq(payment_method_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(update_payment_method.into())
    }
}
