//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "shopping_carts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub cart_id: i32,
    pub customer_id: i32,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::cart_items::Entity")]
    CartItems,
    #[sea_orm(
        belongs_to = "super::customers::Entity",
        from = "(Column::CustomerId, Column::CustomerId)",
        to = "(super::customers::Column::CustomerId, super::customers::Column::CustomerId)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Customers,
}

impl Related<super::cart_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CartItems.def()
    }
}

impl Related<super::customers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
