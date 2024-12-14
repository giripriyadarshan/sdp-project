pub mod addresses;
pub mod bills;
pub mod carts;
pub mod orders;
pub mod payments;
pub mod products;
pub mod user;

pub mod order_und_pagination {
    use async_graphql::{Enum, InputObject, SimpleObject};

    #[derive(InputObject)]
    pub struct Pagination {
        pub page: u64,
        pub page_size: u64,
    }

    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    pub enum OrderByColumn {
        Date,
        Amount,
    }

    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    pub enum OrderByOrder {
        Asc,
        Desc,
    }

    #[derive(InputObject)]
    pub struct OrderBy {
        pub column: OrderByColumn,
        pub order: OrderByOrder,
    }

    #[derive(InputObject)]
    pub struct OrderAndPagination {
        pub order_by: OrderBy,
        pub pagination: Pagination,
    }

    #[derive(SimpleObject)]
    pub struct PageInfo {
        pub total_pages: u64,
        pub total_items: u64,
    }
}
