mod addresses_objects;
mod carts_objects;
mod orders_objects;
mod payments_objects;
mod products_objects;
pub mod schema;
mod users_objects;

pub mod macros {
    macro_rules! role_guard {
        ($($role:expr),*) => {
            RoleGuard::new(vec![$($role),*])
        };
    }

    pub(crate) use role_guard;
}
