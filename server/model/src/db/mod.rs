//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use super::*;

use chrono::Utc;
use futures::future;
use sea_orm::{
    prelude::DateTimeUtc, sea_query::Expr, DbBackend, DeleteResult, DeriveEntityModel,
    FromQueryResult, Order, QueryOrder, QuerySelect, Set, Statement,
};

pub mod area;
pub mod device;
pub mod geofence;
pub mod geofence_project;
pub mod gym;
pub mod instance;
pub mod pokestop;
pub mod prelude;
pub mod project;
pub mod sea_orm_active_enums;
pub mod spawnpoint;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct NameId {
    id: u32,
    name: String,
}

#[derive(FromQueryResult)]
pub struct NameType {
    pub name: String,
    pub instance_type: self::sea_orm_active_enums::Type,
}

#[derive(Serialize, Deserialize)]
pub struct NameTypeId {
    pub id: u32,
    pub name: String,
    pub r#type: self::sea_orm_active_enums::Type,
}

#[derive(Serialize, Deserialize, FromQueryResult)]
pub struct NoFence {
    pub id: u32,
    pub name: String,
    pub mode: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Debug, FromQueryResult)]
pub struct AreaRef {
    pub id: u32,
    pub name: String,
    pub has_geofence: bool,
    pub has_pokemon: bool,
    pub has_quest: bool,
    pub has_fort: bool,
}

#[derive(Debug, FromQueryResult, Serialize, Deserialize, Clone)]
pub struct Total {
    pub total: i32,
}

#[derive(Debug, Serialize)]
pub struct PaginateResults<T> {
    pub results: Vec<(T, Vec<NameId>)>,
    total: usize,
    has_next: bool,
    has_prev: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromQueryResult)]
pub struct Spawnpoint<T = f64>
where
    T: Float,
{
    pub lat: T,
    pub lon: T,
    pub despawn_sec: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RdmInstanceArea {
    Leveling(api::point_struct::PointStruct),
    Single(api::single_struct::SingleStruct),
    Multi(api::multi_struct::MultiStruct),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdmInstance {
    pub area: RdmInstanceArea,
    pub radius: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericData<T = f64>
where
    T: Float,
{
    pub i: String,
    pub p: [T; 2],
}

impl<T> GenericData<T>
where
    T: Float,
{
    pub fn new(i: String, lat: T, lon: T) -> Self {
        GenericData { i, p: [lat, lon] }
    }
}

impl api::ToPointArray for GenericData {
    fn to_point_array(self) -> api::point_array::PointArray {
        self.p
    }
}
impl api::ToPointStruct for GenericData {
    fn to_struct(self) -> api::point_struct::PointStruct {
        api::point_struct::PointStruct {
            lat: self.p[0],
            lon: self.p[1],
        }
    }
}

impl api::ToSingleVec for Vec<GenericData> {
    fn to_single_vec(self) -> api::single_vec::SingleVec {
        self.into_iter()
            .map(|p| api::ToPointArray::to_point_array(p))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InstanceParsing {
    Feature(Feature),
    Rdm(RdmInstance),
}
