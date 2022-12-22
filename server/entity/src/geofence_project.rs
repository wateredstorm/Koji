//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "geofence_project")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub geofence_id: u32,
    pub project_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::geofence::Entity",
        from = "Column::GeofenceId",
        to = "super::geofence::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Geofence,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Project,
}

impl Related<super::geofence::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Geofence.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
