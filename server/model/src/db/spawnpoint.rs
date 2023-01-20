//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use super::*;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "spawnpoint")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub updated: u32,
    pub last_seen: u32,
    pub despawn_sec: Option<u16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct Query;

impl Query {
    pub async fn all(conn: &DatabaseConnection, last_seen: u32) -> Result<Vec<GenericData>, DbErr> {
        let items = spawnpoint::Entity::find()
            .select_only()
            .column(spawnpoint::Column::Lat)
            .column(spawnpoint::Column::Lon)
            .column(spawnpoint::Column::DespawnSec)
            .limit(2_000_000)
            .filter(spawnpoint::Column::LastSeen.gt(last_seen))
            .into_model::<Spawnpoint>()
            .all(conn)
            .await?;
        Ok(utils::normalize::spawnpoint(items))
    }

    pub async fn bound(
        conn: &DatabaseConnection,
        payload: &api::args::BoundsArg,
    ) -> Result<Vec<GenericData>, DbErr> {
        let items = spawnpoint::Entity::find()
            .select_only()
            .column(spawnpoint::Column::Lat)
            .column(spawnpoint::Column::Lon)
            .column(spawnpoint::Column::DespawnSec)
            .filter(spawnpoint::Column::Lat.between(payload.min_lat, payload.max_lat))
            .filter(spawnpoint::Column::Lon.between(payload.min_lon, payload.max_lon))
            .filter(
                Column::Updated.gt(if let Some(last_seen) = payload.last_seen {
                    last_seen
                } else {
                    0
                }),
            )
            .limit(2_000_000)
            .into_model::<Spawnpoint<f64>>()
            .all(conn)
            .await?;
        Ok(utils::normalize::spawnpoint(items))
    }

    pub async fn area(
        conn: &DatabaseConnection,
        area: &FeatureCollection,
        last_seen: u32,
    ) -> Result<Vec<GenericData>, DbErr> {
        let items = spawnpoint::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::MySql,
                format!(
                    "SELECT lat, lon, despawn_sec FROM spawnpoint WHERE last_seen >= {} AND ({}) LIMIT 2000000",
                    last_seen,
                    utils::sql_raw(area)
                )
                .as_str(),
                vec![],
            ))
            .into_model::<Spawnpoint>()
            .all(conn)
            .await?;
        Ok(utils::normalize::spawnpoint(items))
    }

    pub async fn stats(
        conn: &DatabaseConnection,
        area: &FeatureCollection,
        last_seen: u32,
    ) -> Result<Total, DbErr> {
        let items = spawnpoint::Entity::find()
            .column_as(spawnpoint::Column::Id.count(), "count")
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::MySql,
                format!(
                    "SELECT COUNT(*) AS total FROM spawnpoint WHERE last_seen >= {} AND ({})",
                    last_seen,
                    utils::sql_raw(area)
                )
                .as_str(),
                vec![],
            ))
            .into_model::<Total>()
            .one(conn)
            .await?;
        Ok(if let Some(item) = items {
            item
        } else {
            Total { total: 0 }
        })
    }
}
