use crate::{
    auth::auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        orders::{Orders, RegisterOrder},
        payments::{PaymentMethods, RegisterPaymentMethod},
        products::{create_product_model, Products, RegisterProduct},
        user::{
            get_customer_id, Customers, LoginUser, RegisterCustomer, RegisterSupplier,
            RegisterUser, Suppliers, Users,
        },
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    prelude::Decimal, ActiveEnum, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, TransactionTrait,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register_user(
        &self,
        ctx: &Context<'_>,
        input: RegisterUser,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{sea_orm_active_enums::UserRole, users};

        if users::Entity::find()
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
        let insert_user = users::Entity::insert(user).exec_with_returning(db).await?;

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
        token: String,
    ) -> Result<Customers, async_graphql::Error> {
        use crate::entity::customers;

        let db = ctx.data::<DatabaseConnection>()?;

        let customer = customers::ActiveModel {
            first_name: Set(input.first_name),
            last_name: Set(input.last_name),
            user_id: Set(Auth::verify_token(&token)?.user_id.parse::<i32>()?),
            ..Default::default()
        };

        let insert_customer = customers::Entity::insert(customer)
            .exec_with_returning(db)
            .await?;

        Ok(insert_customer.into())
    }

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_supplier(
        &self,
        ctx: &Context<'_>,
        input: RegisterSupplier,
        token: String,
    ) -> Result<Suppliers, async_graphql::Error> {
        use crate::entity::suppliers;

        let db = ctx.data::<DatabaseConnection>()?;

        let supplier = suppliers::ActiveModel {
            user_id: Set(Auth::verify_token(&token)?.user_id.parse::<i32>()?),
            contact_phone: Set(input.contact_phone),
            ..Default::default()
        };

        let insert_supplier = suppliers::Entity::insert(supplier)
            .exec_with_returning(db)
            .await?;

        Ok(insert_supplier.into())
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        login_details: LoginUser,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::users;

        let db = ctx.data::<DatabaseConnection>()?;

        let user: Users = users::Entity::find()
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

    // needs a role_guard to check if the user is a supplier as even a customer has a valid token
    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn register_product(
        &self,
        ctx: &Context<'_>,
        input: RegisterProduct,
        token: String,
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;
        let product = create_product_model(input, token)?;
        let insert_product = products::Entity::insert(product)
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
        token: String,
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;
        let product = create_product_model(input, token)?;
        let update_product = products::Entity::update(product)
            .filter(products::Column::ProductId.eq(product_id))
            .exec(db)
            .await?;
        Ok(update_product.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_order(
        &self,
        ctx: &Context<'_>,
        input: RegisterOrder,
        token: String,
    ) -> Result<Orders, async_graphql::Error> {
        use crate::entity::{order_items, orders, products};
        let db = ctx.data::<DatabaseConnection>()?;
        let txn = db.begin().await?;

        let customer_id = get_customer_id(&db, &token).await?;

        let discount_id = match &input.discount_code {
            Some(discount_code) => {
                let discount = products::Entity::find()
                    .filter(products::Column::Name.eq(discount_code))
                    .one(db)
                    .await
                    .map_err(|_| "Discount not found")?
                    .map(|product| product.product_id);
                discount
            }
            None => None,
        };

        let mut total_amount: f64 = 0.0;
        for item in &input.order_items {
            let product: products::Model = products::Entity::find_by_id(item.product_id)
                .one(db)
                .await
                .map_err(|_| "Product not found")?
                .unwrap();
            total_amount += product.base_price.to_string().parse::<f64>()? * item.quantity as f64;
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

        let insert_order = orders::Entity::insert(order)
            .exec_with_returning(&txn)
            .await?;

        for item in &input.order_items {
            let product: products::Model = products::Entity::find_by_id(item.product_id)
                .one(&txn)
                .await?
                .unwrap();

            let product_base_price = product.base_price;

            let product: products::ActiveModel = products::ActiveModel {
                stock_quantity: Set(product.stock_quantity - item.quantity),
                ..product.into()
            };

            products::Entity::update(product)
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
            order_items::Entity::insert(order_item).exec(&txn).await?;
        }

        txn.commit().await?;

        Ok(insert_order.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_payment_method(
        &self,
        ctx: &Context<'_>,
        input: RegisterPaymentMethod,
        token: String,
    ) -> Result<PaymentMethods, async_graphql::Error> {
        use crate::entity::{card_types, payment_methods, sea_orm_active_enums::PaymentMethodType};
        let db = ctx.data::<DatabaseConnection>()?;
        let txn = db.begin().await?;

        let customer_id = get_customer_id(&db, &token).await?;
        let is_default: Option<bool> = Some(input.is_default.unwrap_or(false));

        let payment_method = match input.payment_type.as_str() {
            "card" => {
                let card_type_id = card_types::Entity::insert(card_types::ActiveModel {
                    name: Set(input
                        .card_type_name
                        .unwrap_or_else(|| "Unknown".to_string())),
                    ..Default::default()
                })
                .exec(&txn)
                .await?
                .last_insert_id;

                payment_methods::ActiveModel {
                    customer_id: Set(customer_id),
                    payment_type: Set(PaymentMethodType::Card),
                    is_default: Set(is_default),
                    card_number: Set(Some(
                        input
                            .card_number
                            .map_or(Err("Card number is required"), Ok)?,
                    )),
                    card_expiration_date: Set(Some(
                        input
                            .card_expiration_date
                            .map_or(Err("Card expiration date is required"), Ok)?,
                    )),
                    card_type_id: Set(Some(card_type_id)),
                    ..Default::default()
                }
            }
            "upi" => payment_methods::ActiveModel {
                customer_id: Set(customer_id),
                payment_type: Set(PaymentMethodType::Upi),
                is_default: Set(is_default),
                upi_id: Set(Some(input.upi_id.map_or(Err("UPI ID is required"), Ok)?)),
                ..Default::default()
            },
            "iban" => payment_methods::ActiveModel {
                customer_id: Set(customer_id),
                payment_type: Set(PaymentMethodType::Iban),
                is_default: Set(is_default),
                iban: Set(Some(input.iban.map_or(Err("IBAN number is required"), Ok)?)),
                ..Default::default()
            },
            "netbanking" => payment_methods::ActiveModel {
                customer_id: Set(customer_id),
                payment_type: Set(PaymentMethodType::Netbanking),
                is_default: Set(is_default),
                bank_name: Set(Some(
                    input.bank_name.map_or(Err("Bank name is required"), Ok)?,
                )),
                account_holder_name: Set(Some(
                    input
                        .account_holder_name
                        .map_or(Err("Account holder name is required"), Ok)?,
                )),
                bank_account_number: Set(Some(
                    input
                        .bank_account_number
                        .map_or(Err("Bank account number is required"), Ok)?,
                )),
                ifsc_code: Set(Some(
                    input.ifsc_code.map_or(Err("IFSC code is required"), Ok)?,
                )),
                ..Default::default()
            },
            _ => return Err("Invalid payment type".into()),
        };

        let insert_payment_method = payment_methods::Entity::insert(payment_method)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(insert_payment_method.into())
    }
}
