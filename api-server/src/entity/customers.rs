//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "customers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    pub registration_date: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::addresses::Entity")]
    Addresses,
    #[sea_orm(has_many = "super::orders::Entity")]
    Orders,
    #[sea_orm(has_many = "super::payment_methods::Entity")]
    PaymentMethods,
    #[sea_orm(has_many = "super::reviews::Entity")]
    Reviews,
    #[sea_orm(has_many = "super::shopping_carts::Entity")]
    ShoppingCarts,
}

impl Related<super::addresses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Addresses.def()
    }
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl Related<super::payment_methods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentMethods.def()
    }
}

impl Related<super::reviews::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reviews.def()
    }
}

impl Related<super::shopping_carts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShoppingCarts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}