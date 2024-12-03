//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub order_id: i32,
    pub customer_id: i32,
    pub order_date: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub total_amount: Decimal,
    pub status: String,
    pub shipping_address_id: i32,
    pub payment_method_id: i32,
    pub discount_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::addresses::Entity",
        from = "Column::ShippingAddressId",
        to = "super::addresses::Column::AddressId",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    Addresses,
    #[sea_orm(has_one = "super::bills::Entity")]
    Bills,
    #[sea_orm(
        belongs_to = "super::customers::Entity",
        from = "(Column::CustomerId, Column::CustomerId)",
        to = "(super::customers::Column::CustomerId, super::customers::Column::CustomerId)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Customers,
    #[sea_orm(
        belongs_to = "super::discounts::Entity",
        from = "Column::DiscountId",
        to = "super::discounts::Column::DiscountId",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Discounts,
    #[sea_orm(has_many = "super::order_items::Entity")]
    OrderItems,
    #[sea_orm(
        belongs_to = "super::payment_methods::Entity",
        from = "Column::PaymentMethodId",
        to = "super::payment_methods::Column::PaymentMethodId",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    PaymentMethods,
}

impl Related<super::addresses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Addresses.def()
    }
}

impl Related<super::bills::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bills.def()
    }
}

impl Related<super::customers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customers.def()
    }
}

impl Related<super::discounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Discounts.def()
    }
}

impl Related<super::order_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderItems.def()
    }
}

impl Related<super::payment_methods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentMethods.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
