//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "order_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub order_item_id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub unit_price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub discount_amount: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::orders::Entity",
        from = "(Column::OrderId, Column::OrderId)",
        to = "(super::orders::Column::OrderId, super::orders::Column::OrderId)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Orders,
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "(Column::ProductId, Column::ProductId, Column::ProductId)",
        to = "(super::products::Column::ProductId, super::products::Column::ProductId, super::products::Column::ProductId)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Products,
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
