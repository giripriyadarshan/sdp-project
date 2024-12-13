use crate::models::addresses::{create_address, Addresses, RegisterAddress};
use crate::{
    auth::auth::{Auth, RoleGuard, ROLE_CUSTOMER, ROLE_SUPPLIER},
    graphql::macros::role_guard,
    models::{
        orders::{Orders, RegisterOrder},
        payments::{create_payment_method, PaymentMethods, RegisterPaymentMethod},
        products::{
            check_if_supplier_owns_product, check_product_exists, create_discount_model,
            create_product_model, create_review_model, Discounts, Products, RegisterDiscount,
            RegisterProduct, RegisterReview, Reviews,
        },
        user::{
            get_customer_supplier_id, Customers, LoginUser, RegisterCustomer, RegisterSupplier,
            RegisterUser, Suppliers, Users,
        },
    },
};
use async_graphql::{Context, Object};
use sea_orm::{
    prelude::Decimal, ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, TransactionTrait,
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
    ) -> Result<Customers, async_graphql::Error> {
        use crate::entity::customers;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

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
    ) -> Result<Suppliers, async_graphql::Error> {
        use crate::entity::suppliers;

        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

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
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;
        let product = create_product_model(input, supplier_id)?;
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
    ) -> Result<Products, async_graphql::Error> {
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;
        check_if_supplier_owns_product(db, supplier_id, product_id).await?;
        let product = create_product_model(input, supplier_id)?;
        let update_product = products::Entity::update(product)
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
        use crate::entity::products;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;

        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, product_id).await?;
        products::Entity::delete_by_id(product_id).exec(db).await?;
        Ok("Product deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_order(
        &self,
        ctx: &Context<'_>,
        input: RegisterOrder,
    ) -> Result<Orders, async_graphql::Error> {
        use crate::entity::{discounts, order_items, orders, products};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let discount_id = match &input.discount_code {
            Some(discount_code) => products::Entity::find()
                .filter(products::Column::Name.eq(discount_code))
                .one(db)
                .await
                .map_err(|_| "Discount not found")?
                .map(|product| product.product_id),
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

        if let Some(discount_id) = discount_id {
            let discount: discounts::Model = discounts::Entity::find_by_id(discount_id)
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

        let insert_order = orders::Entity::insert(order)
            .exec_with_returning(&txn)
            .await?;

        for item in &input.order_items {
            let product: products::Model = products::Entity::find_by_id(item.product_id)
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

    #[graphql(guard = "role_guard!(ROLE_SUPPLIER)")]
    async fn update_order_status(
        &self,
        ctx: &Context<'_>,
        order_id: i32,
        status: String,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::orders;
        let db = ctx.data::<DatabaseConnection>()?;
        let txn = db.begin().await?;

        let order: orders::Model = orders::Entity::find_by_id(order_id)
            .one(&txn)
            .await
            .map_err(|_| "Order not found")?
            .unwrap();

        let update_order = orders::ActiveModel {
            status: Set(status),
            ..order.into()
        };

        orders::Entity::update(update_order)
            .filter(orders::Column::OrderId.eq(order_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok("Order status updated".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_payment_method(
        &self,
        ctx: &Context<'_>,
        input: RegisterPaymentMethod,
    ) -> Result<PaymentMethods, async_graphql::Error> {
        use crate::entity::payment_methods;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;
        let is_default: Option<bool> = Some(input.is_default.unwrap_or(false));

        let payment_method = create_payment_method(customer_id, is_default, input, &txn).await?;

        let insert_payment_method = payment_methods::Entity::insert(payment_method)
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
        use crate::entity::payment_methods;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;
        let is_default: Option<bool> = Some(input.is_default.unwrap_or(false));

        let payment_method = create_payment_method(customer_id, is_default, input, &txn).await?;

        let update_payment_method = payment_methods::Entity::update(payment_method)
            .filter(payment_methods::Column::PaymentMethodId.eq(payment_method_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(update_payment_method.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn add_to_cart(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        quantity: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{cart_items, shopping_carts};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = match shopping_carts::Entity::find()
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

        cart_items::Entity::insert(cart_item).exec(&txn).await?;
        txn.commit().await?;

        Ok("Product added to cart".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_cart_item_quantity(
        &self,
        ctx: &Context<'_>,
        product_id: i32,
        quantity: i32,
        cart_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{cart_items, shopping_carts};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = shopping_carts::Entity::find_by_id(cart_id)
            .one(&txn)
            .await?
            .ok_or("Cart not found")?;

        let cart_item = cart_items::Entity::find()
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
        use crate::entity::{cart_items, shopping_carts};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        check_product_exists(&txn, product_id).await?;

        let cart = match shopping_carts::Entity::find()
            .filter(shopping_carts::Column::CustomerId.eq(customer_id))
            .one(&txn)
            .await?
        {
            Some(cart) => cart,
            None => {
                return Err("Cart not found".into());
            }
        };

        let cart_item = cart_items::Entity::find()
            .filter(cart_items::Column::CartId.eq(cart.cart_id))
            .filter(cart_items::Column::ProductId.eq(product_id))
            .one(&txn)
            .await?
            .ok_or("Product not found in cart")?;

        cart_item.delete(&txn).await?;

        txn.commit().await?;

        Ok("Product removed from cart".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_review(
        &self,
        ctx: &Context<'_>,
        input: RegisterReview,
    ) -> Result<Reviews, async_graphql::Error> {
        use crate::entity::{order_items, orders, reviews};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        // check if customer has ordered the product
        if order_items::Entity::find()
            .inner_join(orders::Entity)
            .filter(orders::Column::CustomerId.eq(customer_id))
            .one(&txn)
            .await?
            .is_none()
        {
            return Err("Customer has not ordered the product".into());
        }

        let review = create_review_model(input, customer_id)?;

        let insert_review = reviews::Entity::insert(review)
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
        use crate::entity::reviews;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let review = create_review_model(input, customer_id)?;

        let update_review = reviews::Entity::update(review)
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
        use crate::entity::reviews;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;

        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let review = reviews::Entity::find()
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
        use crate::entity::discounts;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, input.product_id).await?;

        let discount = create_discount_model(input)?;
        let insert_discount = discounts::Entity::insert(discount)
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
        use crate::entity::discounts;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, input.product_id).await?;

        let discount = create_discount_model(input)?;
        let update_discount = discounts::Entity::update(discount)
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
        use crate::entity::discounts;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let supplier_id = get_customer_supplier_id(db, &token, ROLE_SUPPLIER).await?;

        check_if_supplier_owns_product(db, supplier_id, product_id).await?;

        discounts::Entity::delete_by_id(discount_id)
            .exec(db)
            .await?;
        Ok("Discount deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn register_address(
        &self,
        ctx: &Context<'_>,
        input: RegisterAddress,
    ) -> Result<Addresses, async_graphql::Error> {
        use crate::entity::{address_types, addresses};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let address_type = address_types::ActiveModel {
            name: Set(input.clone().address_type),
            ..Default::default()
        };

        let address_type_id = address_types::Entity::insert(address_type)
            .exec(&txn)
            .await?
            .last_insert_id;

        let address = create_address(input, customer_id, address_type_id, &txn).await?;

        let insert_address = addresses::Entity::insert(address)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(insert_address.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_address(
        &self,
        ctx: &Context<'_>,
        address_id: i32,
        address_type_id: i32,
        input: RegisterAddress,
    ) -> Result<Addresses, async_graphql::Error> {
        use crate::entity::addresses;
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let address = create_address(input, customer_id, address_type_id, &txn).await?;

        let update_address = addresses::Entity::update(address)
            .filter(addresses::Column::AddressId.eq(address_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(update_address.into())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn delete_address(
        &self,
        ctx: &Context<'_>,
        address_id: i32,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{address_types, addresses};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let address = addresses::Entity::find()
            .filter(addresses::Column::AddressId.eq(address_id))
            .one(&txn)
            .await?
            .ok_or("Address not found")?;

        if address.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        if address.is_default.unwrap_or(false) {
            return Err("Cannot delete default address".into());
        }

        let address_type = address_types::Entity::find_by_id(address.address_type_id.unwrap())
            .one(&txn)
            .await?
            .ok_or("Address type not found")?;

        address_type.delete(&txn).await?;
        address.delete(&txn).await?;

        txn.commit().await?;

        Ok("Address deleted".to_string())
    }

    #[graphql(guard = "role_guard!(ROLE_CUSTOMER)")]
    async fn update_address_type(
        &self,
        ctx: &Context<'_>,
        address_type_id: i32,
        name: String,
    ) -> Result<String, async_graphql::Error> {
        use crate::entity::{address_types, addresses};
        let db = ctx.data::<DatabaseConnection>()?;
        let token = ctx
            .data_opt::<String>()
            .ok_or("No authorization token found")?;
        let txn = db.begin().await?;
        let customer_id = get_customer_supplier_id(db, &token, ROLE_CUSTOMER).await?;

        let address_type = address_types::Entity::find()
            .filter(address_types::Column::AddressTypeId.eq(address_type_id))
            .one(&txn)
            .await?
            .ok_or("Address type not found")?;

        let address = addresses::Entity::find()
            .filter(addresses::Column::AddressTypeId.eq(address_type_id))
            .one(&txn)
            .await?
            .ok_or("Address not found")?;

        if address.customer_id != customer_id {
            return Err("Unauthorized".into());
        }

        let mut address_type: address_types::ActiveModel = address_type.into();
        address_type.name = Set(name);

        address_type.update(&txn).await?;

        txn.commit().await?;

        Ok("Address type updated".to_string())
    }
}
