//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "area")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    #[sea_orm(unique)]
    pub name: String,
    pub pokemon_mode_workers: u32,
    #[sea_orm(column_type = "Custom(\"MEDIUMTEXT\".to_owned())", nullable)]
    pub pokemon_mode_route: Option<String>,
    pub fort_mode_workers: u32,
    #[sea_orm(column_type = "Custom(\"MEDIUMTEXT\".to_owned())", nullable)]
    pub fort_mode_route: Option<String>,
    pub quest_mode_workers: u32,
    #[sea_orm(column_type = "Text", nullable)]
    pub quest_mode_hours: Option<String>,
    pub quest_mode_max_login_queue: Option<u16>,
    #[sea_orm(column_type = "Text", nullable)]
    pub geofence: Option<String>,
    pub enable_quests: i8,
    #[sea_orm(column_type = "Custom(\"MEDIUMTEXT\".to_owned())", nullable)]
    pub quest_mode_route: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}