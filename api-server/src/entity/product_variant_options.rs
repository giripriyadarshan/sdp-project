//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product_variant_options")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub option_id: i32,
    pub product_id: i32,
    pub option_name: String,
    pub option_value: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub price_adjustment: Option<Decimal>,
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
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
