mod mutation_root;
mod query_root;
pub mod schema;

pub mod macros {
    macro_rules! role_guard {
        ($($role:expr),*) => {
            RoleGuard::new(vec![$($role),*])
        };
    }

    pub(crate) use role_guard;
}
