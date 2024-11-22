//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "cart_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub cart_item_id: i32,
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "(Column::ProductId, Column::ProductId)",
        to = "(super::products::Column::ProductId, super::products::Column::ProductId)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Products,
    #[sea_orm(
        belongs_to = "super::shopping_carts::Entity",
        from = "Column::CartId",
        to = "super::shopping_carts::Column::CartId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ShoppingCarts,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}

impl Related<super::shopping_carts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShoppingCarts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
