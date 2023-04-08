//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use std::{collections::HashMap, str::FromStr};

use crate::{
    api::{args::ApiQueryArgs, GeoFormats, ToCollection},
    error::ModelError,
    utils::{
        json::{determine_category_by_value, JsonToModel},
        json_related_sort, name_modifier, parse_order,
    },
};

use super::{geofence_property::FullPropertyModel, sea_orm_active_enums::Type, *};

use geojson::{GeoJson, Geometry};
use sea_orm::{entity::prelude::*, UpdateResult};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "geofence")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    #[sea_orm(unique)]
    pub name: String,
    pub parent: Option<u32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub mode: Type,
    pub geometry: Json,
    pub geo_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::Parent",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::project::Entity")]
    Project,
    #[sea_orm(has_many = "super::geofence_property::Entity")]
    GeofenceProperty,
    #[sea_orm(has_many = "super::route::Entity")]
    Route,
}

impl Related<project::Entity> for Entity {
    fn to() -> RelationDef {
        geofence_project::Relation::Project.def()
    }
    fn via() -> Option<RelationDef> {
        Some(geofence_project::Relation::Geofence.def().rev())
    }
}

impl Related<super::geofence_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GeofenceProperty.def()
    }
}

impl Related<super::route::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Route.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Serialize, Deserialize, FromQueryResult)]
pub struct GeofenceNoGeometry {
    pub id: u32,
    pub name: String,
    pub mode: Type,
    pub geo_type: String,
    pub parent: Option<u32>,
    // pub created_at: DateTimeUtc,
    // pub updated_at: DateTimeUtc,
}

impl GeofenceNoGeometry {
    fn to_json(self) -> Json {
        json!(self)
    }
}

impl Model {
    fn to_json(self) -> Json {
        json!(self)
    }

    async fn get_parent_name(&self, db: &DatabaseConnection) -> Result<Option<String>, ModelError> {
        if let Some(parent_id) = self.parent {
            if let Some(parent) = geofence::Entity::find_by_id(parent_id).one(db).await? {
                let found_name = if let Some(name) = parent
                    .get_related_properties()
                    .into_model::<FullPropertyModel>()
                    .all(db)
                    .await?
                    .iter()
                    .find(|parent_prop| parent_prop.name == "name")
                {
                    let parsed = name.parse_db_value(&parent);
                    if let Some(parsed) = parsed.as_str() {
                        parsed.to_string()
                    } else {
                        parent.name
                    }
                } else {
                    parent.name
                };
                return Ok(Some(found_name));
            }
        }
        Ok(None)
    }

    async fn to_feature(
        self,
        db: &DatabaseConnection,
        args: &ApiQueryArgs,
    ) -> Result<Feature, ModelError> {
        let mut properties = self
            .get_related_properties()
            .into_model::<FullPropertyModel>()
            .all(db)
            .await?
            .into_iter()
            .map(|prop| prop.parse_db_value(&self))
            .collect::<Vec<serde_json::Value>>();
        let parent_name = self.get_parent_name(db).await?;

        if args.internal.is_some() {
            properties.push(json!({ "name": "__id", "value": self.id }));
            properties.push(json!({ "name": "__name", "value": self.name }));
            properties.push(json!({ "name": "__mode", "value": self.mode.to_value() }));
        }
        if args.name.is_some() {
            properties.push(json!({ "name": "name", "value": self.name }));
        }
        if args.id.is_some() {
            properties.push(json!({ "name": "id", "value": self.id }));
        }
        if args.mode.is_some() {
            properties.push(json!({ "name": "mode", "value": self.mode.to_value() }));
        }
        if args.parent.is_some() {
            properties.push(json!({ "name": "parent", "value": parent_name }));
        }
        if args.group.is_some() {
            properties.push(json!({ "name": "group", "value": parent_name }));
        }

        let mut feature = Feature {
            geometry: Some(Geometry::from_json_value(self.geometry)?),
            ..Default::default()
        };
        for prop in properties.iter() {
            let key = prop.get("name");
            let val = prop.get("value");
            if let Some(key) = key {
                if let Some(key) = key.as_str() {
                    if key.eq("parent") {
                        feature.set_property(
                            if args.group.is_some() { "group" } else { key },
                            parent_name.clone(),
                        )
                    } else {
                        feature.set_property(key, val.unwrap().clone())
                    }
                }
            }
        }

        if let Some(geofence_name) = feature.property("name") {
            if let Some(geofence_name) = geofence_name.as_str() {
                feature.set_property(
                    "name",
                    name_modifier(geofence_name.to_string(), args, parent_name),
                );
            }
        }

        if args.internal.is_some() {
            feature.id = Some(geojson::feature::Id::String(format!(
                "{}__{}__KOJI",
                self.id,
                self.mode.to_value()
            )));
        }
        Ok(feature)
    }

    fn get_related_projects(&self) -> Select<project::Entity> {
        self.find_related(project::Entity)
            .select_only()
            .column(project::Column::Id)
            .column(project::Column::Name)
    }

    fn get_related_routes(&self) -> Select<route::Entity> {
        self.find_related(route::Entity)
            .select_only()
            .column(route::Column::Id)
            .column(route::Column::Name)
    }

    fn get_related_properties(&self) -> Select<geofence_property::Entity> {
        self.find_related(geofence_property::Entity)
            .join(
                sea_orm::JoinType::Join,
                geofence_property::Relation::Property.def(),
            )
            .select_only()
            .column(geofence_property::Column::Id)
            .column(geofence_property::Column::PropertyId)
            .column(geofence_property::Column::GeofenceId)
            .column(geofence_property::Column::Value)
            .column(property::Column::Name)
            .column(property::Column::Category)
            .filter(geofence_property::Column::GeofenceId.eq(self.id))
    }
}

impl VecToJson for Vec<Model> {
    fn to_json(self) -> Vec<Json> {
        self.into_iter().map(|model| model.to_json()).collect()
    }
}

impl VecToJson for Vec<GeofenceNoGeometry> {
    fn to_json(self) -> Vec<Json> {
        self.into_iter().map(|model| model.to_json()).collect()
    }
}

pub struct Query;

impl Query {
    /// Returns a single Geofence model and it's related projects as tuple
    pub async fn get_one(db: &DatabaseConnection, id: String) -> Result<Model, ModelError> {
        let record = match id.parse::<u32>() {
            Ok(id) => Entity::find_by_id(id).one(db).await?,
            Err(_) => Entity::find().filter(Column::Name.eq(id)).one(db).await?,
        };
        if let Some(record) = record {
            Ok(record)
        } else {
            Err(ModelError::Geofence("Does not exist".to_string()))
        }
    }

    pub async fn get_one_json(db: &DatabaseConnection, id: String) -> Result<Json, ModelError> {
        match Query::get_one(db, id).await {
            Ok(record) => Ok(json!(record)),
            Err(err) => Err(err),
        }
    }

    pub async fn get_one_json_with_related(
        db: &DatabaseConnection,
        id: String,
    ) -> Result<Json, ModelError> {
        match Query::get_one(db, id).await {
            Ok(record) => {
                let mut json = json!(record);
                let json = json.as_object_mut().unwrap();
                if json.contains_key("area") {
                    json.remove("area");
                }
                json.insert(
                    "projects".to_string(),
                    json!(record
                        .get_related_projects()
                        .into_model::<NameId>()
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|p| p.id)
                        .collect::<Vec<u32>>()),
                );
                json.insert(
                    "routes".to_string(),
                    json!(record.get_related_routes().into_json().all(db).await?),
                );
                json.insert(
                    "properties".to_string(),
                    json!(record
                        .get_related_properties()
                        .into_model::<FullPropertyModel>()
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|prop| prop.parse_db_value(&record))
                        .collect::<Vec<Json>>()),
                );
                Ok(json!(json))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_one_feature(
        db: &DatabaseConnection,
        id: String,
        args: &ApiQueryArgs,
    ) -> Result<Feature, ModelError> {
        match Query::get_one(db, id).await {
            Ok(record) => record.to_feature(db, args).await,
            Err(err) => Err(err),
        }
    }

    /// Returns all Geofence models in the db
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(db).await
    }

    /// Returns all Geofence models in the db
    pub async fn get_all_json(db: &DatabaseConnection) -> Result<Vec<Json>, DbErr> {
        Ok(Query::get_all(db).await?.to_json())
    }

    /// Returns all geofence models as a FeatureCollection,
    pub async fn get_all_collection(
        db: &DatabaseConnection,
        args: &ApiQueryArgs,
    ) -> Result<FeatureCollection, ModelError> {
        let results = Query::get_all(db).await?;

        let results = future::try_join_all(
            results
                .into_iter()
                .map(|result| result.to_feature(db, args)),
        )
        .await?;

        Ok(results.to_collection(None, None))
    }

    /// Returns all Geofence models in the db without their features
    pub async fn get_all_no_fences(
        db: &DatabaseConnection,
    ) -> Result<Vec<GeofenceNoGeometry>, DbErr> {
        Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .column(Column::Mode)
            .column(Column::GeoType)
            .column(Column::Parent)
            .order_by(Column::Name, Order::Asc)
            .into_model::<GeofenceNoGeometry>()
            .all(db)
            .await
    }

    pub async fn get_json_cache(db: &DatabaseConnection) -> Result<Vec<sea_orm::JsonValue>, DbErr> {
        Ok(Query::get_all_no_fences(db).await?.to_json())
    }

    /// Returns paginated Geofence models
    pub async fn paginate(
        db: &DatabaseConnection,
        page: u64,
        posts_per_page: u64,
        order: String,
        sort_by: String,
        q: String,
    ) -> Result<PaginateResults<Vec<Json>>, DbErr> {
        let column = Column::from_str(&sort_by).unwrap_or(Column::Name);

        let paginator = Entity::find()
            .order_by(column, parse_order(&order))
            .filter(Column::Name.like(format!("%{}%", q).as_str()))
            .paginate(db, posts_per_page);
        let total = paginator.num_items_and_pages().await?;

        let results = paginator.fetch_page(page).await?;

        let projects = future::try_join_all(
            results
                .iter()
                .map(|result| result.get_related_projects().into_json().all(db)),
        )
        .await?;

        let properties = future::try_join_all(
            results
                .iter()
                .map(|result| result.get_related_properties().into_json().all(db)),
        )
        .await?;

        let routes = future::try_join_all(
            results
                .iter()
                .map(|result| result.get_related_routes().into_json().all(db)),
        )
        .await?;

        let mut results: Vec<Json> = results
            .into_iter()
            .enumerate()
            .map(|(i, fence)| {
                json!({
                    "id": fence.id,
                    "name": fence.name,
                    "mode": fence.mode,
                    "geo_type": fence.geo_type,
                    "parent": fence.parent,
                    // "created_at": fence.created_at,
                    // "updated_at": fence.updated_at,
                    "projects": projects[i],
                    "properties": properties[i],
                    "routes": routes[i],
                })
            })
            .collect();

        if sort_by.contains("length") {
            json_related_sort(&mut results, &sort_by.replace(".length", ""), order);
        }

        Ok(PaginateResults {
            results,
            total: total.number_of_items,
            has_prev: total.number_of_pages == page + 1,
            has_next: page + 1 < total.number_of_pages,
        })
    }

    pub async fn update_related_route_names(
        conn: &DatabaseConnection,
        old_model: &Model,
        new_name: String,
    ) -> Result<UpdateResult, DbErr> {
        route::Entity::update_many()
            .col_expr(route::Column::Name, Expr::value(new_name))
            .filter(route::Column::GeofenceId.eq(old_model.id.to_owned()))
            .filter(route::Column::Name.eq(old_model.name.to_owned()))
            .exec(conn)
            .await
    }

    pub async fn upsert_related_properties(
        db: &DatabaseConnection,
        json: &serde_json::Value,
        geofence_id: u32,
    ) -> Result<(), ModelError> {
        if let Some(properties) = json.get("properties") {
            if let Some(properties) = properties.as_array() {
                let mut existing = vec![];
                let mut new_props = vec![];
                properties.iter().for_each(|property| {
                    if let Some(prop_map) = property.as_object() {
                        if prop_map.contains_key("property_id") {
                            existing.push(property.clone())
                        } else {
                            new_props.push(property.clone())
                        }
                    }
                });
                let upserted_props = future::try_join_all(
                    new_props
                        .clone()
                        .into_iter()
                        .map(|result| property::Query::upsert(db, 0, result)),
                )
                .await?;

                upserted_props
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, prop)| {
                        existing.push(json!({
                            "value": new_props[i]["value"],
                            "property_id": prop.id,
                            "geofence_id": geofence_id,
                        }))
                    });

                geofence_property::Query::update_properties_by_geofence(
                    db,
                    &existing,
                    Some(geofence_id),
                )
                .await?;
            };
        };
        Ok(())
    }

    pub async fn upsert_related_projects(
        db: &DatabaseConnection,
        json: &serde_json::Value,
        geofence_id: u32,
    ) -> Result<(), DbErr> {
        if let Some(projects) = json.get("projects") {
            if let Some(projects) = projects.as_array() {
                geofence_project::Query::upsert_related_by_geofence_id(db, projects, geofence_id)
                    .await?;
            };
        };
        Ok(())
    }

    /// Updates or creates a Geofence model, returns a model struct
    pub async fn upsert(db: &DatabaseConnection, id: u32, json: Json) -> Result<Model, ModelError> {
        let mut json = json;
        let old_model = Entity::find_by_id(id).one(db).await?;
        let mut new_model = json.to_geofence()?;

        let name = new_model.name.as_ref();

        let old_model = if let Some(old_model) = old_model {
            Ok(old_model)
        } else {
            Query::get_one(db, name.to_string()).await
        };
        let model = if let Ok(old_model) = old_model {
            if old_model.name.ne(name) {
                Query::update_related_route_names(db, &old_model, name.clone()).await?;
            };
            new_model.id = Set(old_model.id);
            new_model.update(db).await?
        } else {
            let model = new_model.insert(db).await?;
            let prop_name_model = geofence_property::Query::add_name_property(db, model.id).await?;
            if let Some(properties) = json["properties"].as_array_mut() {
                properties.push(json!({
                    "property_id": prop_name_model.property_id,
                    "geofence_id": prop_name_model.geofence_id,
                }))
            }
            model
        };
        Query::upsert_related_projects(db, &json, model.id).await?;
        Query::upsert_related_properties(db, &json, model.id).await?;
        Ok(model)
    }

    /// Updates or creates a Geofence model, returns a json
    pub async fn upsert_json_return(
        db: &DatabaseConnection,
        id: u32,
        json: Json,
    ) -> Result<Json, ModelError> {
        let result = Query::upsert(db, id, json).await?;
        Ok(result.to_json())
    }

    /// Deletes a Geofence model from db
    pub async fn delete(db: &DatabaseConnection, id: u32) -> Result<DeleteResult, DbErr> {
        let record = Entity::delete_by_id(id).exec(db).await?;
        Ok(record)
    }

    async fn upsert_feature(conn: &DatabaseConnection, feat: Feature) -> Result<Model, ModelError> {
        let mut new_map = HashMap::<&str, serde_json::Value>::new();

        let id = if let Some(id) = feat.property("__id") {
            if let Some(id) = id.as_u64() {
                id
            } else {
                0
            }
        } else {
            0
        } as u32;

        if let Some(name) = feat.property("__name") {
            if let Some(name) = name.as_str() {
                new_map.insert("name", serde_json::Value::String(name.to_string()));
            } else {
                return Err(ModelError::Geofence("Name is invalid string".to_string()));
            }
        } else {
            return Err(ModelError::Geofence("Missing name property".to_string()));
        };

        if let Some(geometry) = feat.geometry.clone() {
            new_map.insert("geometry", GeoJson::Geometry(geometry).to_json_value());
        } else {
            return Err(ModelError::Geofence(
                "Did not find valid Geometry".to_string(),
            ));
        };

        if let Some(mode) = feat.property("__mode") {
            if let Some(mode) = mode.as_str() {
                new_map.insert("mode", serde_json::Value::String(mode.to_string()));
            }
        };
        if let Some(projects) = feat.property("__projects") {
            new_map.insert("projects", projects.clone());
        };

        let properties = feat
            .properties_iter()
            .filter_map(|(k, v)| {
                if k.starts_with("__") {
                    None
                } else {
                    let (category, value) = determine_category_by_value(k, v.clone(), &new_map);
                    Some(json!({ "name": k, "value": value, "category": category }))
                }
            })
            .collect::<Vec<serde_json::Value>>();
        new_map.insert("properties", json!(properties));

        Query::upsert(conn, id, json!(new_map)).await
    }

    pub async fn upsert_from_geometry(
        conn: &DatabaseConnection,
        area: GeoFormats,
    ) -> Result<(), ModelError> {
        match area {
            GeoFormats::Feature(feat) => {
                Query::upsert_feature(conn, feat).await?;
            }
            feat => {
                let fc = match feat {
                    GeoFormats::FeatureCollection(fc) => fc,
                    geometry => geometry.to_collection(None, None),
                };
                for feat in fc.into_iter() {
                    Query::upsert_feature(conn, feat).await?;
                }
            }
        };
        Ok(())
    }

    /// Returns all geofence models, as models, that are related to the specified project
    pub async fn by_project(
        db: &DatabaseConnection,
        project_name: String,
    ) -> Result<Vec<Json>, DbErr> {
        match project_name.parse::<u32>() {
            Ok(id) => {
                Entity::find()
                    .order_by(Column::Name, Order::Asc)
                    .left_join(project::Entity)
                    .filter(project::Column::Id.eq(id))
                    .select_only()
                    .column(Column::Id)
                    .column(Column::Name)
                    .column(Column::Mode)
                    .column(Column::Parent)
                    .into_json()
                    .all(db)
                    .await
            }
            Err(_) => {
                Entity::find()
                    .order_by(Column::Name, Order::Asc)
                    .left_join(project::Entity)
                    .filter(project::Column::Name.eq(project_name))
                    .select_only()
                    .column(Column::Id)
                    .column(Column::Name)
                    .column(Column::Mode)
                    .column(Column::Parent)
                    .into_json()
                    .all(db)
                    .await
            }
        }
    }
    /// Returns all geofence models, as features, that are related to the specified project
    pub async fn project_as_feature(
        db: &DatabaseConnection,
        project_name: String,
        args: &ApiQueryArgs,
    ) -> Result<Vec<Feature>, ModelError> {
        let items = match project_name.parse::<u32>() {
            Ok(id) => {
                Entity::find()
                    .order_by(Column::Name, Order::Asc)
                    .left_join(project::Entity)
                    .filter(project::Column::Id.eq(id))
                    .all(db)
                    .await?
            }
            Err(_) => {
                Entity::find()
                    .order_by(Column::Name, Order::Asc)
                    .left_join(project::Entity)
                    .filter(project::Column::Name.eq(project_name))
                    .all(db)
                    .await?
            }
        };

        let items =
            future::try_join_all(items.into_iter().map(|result| result.to_feature(db, args)))
                .await?;

        Ok(items)
    }

    pub async fn search(db: &DatabaseConnection, search: String) -> Result<Vec<Json>, DbErr> {
        Ok(Entity::find()
            .filter(Column::Name.like(format!("%{}%", search).as_str()))
            .into_json()
            .all(db)
            .await?)
    }

    pub async fn assign(
        db: &DatabaseConnection,
        id: u32,
        property: String,
        payload: serde_json::Value,
    ) -> Result<Model, ModelError> {
        let column = Column::from_str(&property);

        if let Ok(column) = column {
            let model = Entity::find_by_id(id).one(db).await?;
            if let Some(model) = model {
                let mut model: ActiveModel = model.into();
                match column {
                    Column::Parent => {
                        model.parent = Set(Some(payload.as_u64().unwrap() as u32));
                    }
                    _ => {}
                }
                let model = model.update(db).await?;
                Ok(model)
            } else {
                Err(ModelError::Geofence("Model not found".to_string()))
            }
        } else {
            Err(ModelError::Geofence("Invalid property".to_string()))
        }
    }
}
