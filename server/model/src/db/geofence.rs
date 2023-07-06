//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use std::{collections::HashMap, str::FromStr, time::Instant};

use crate::{
    api::{
        args::{AdminReqParsed, ApiQueryArgs, UnknownId},
        collection::Default,
        GeoFormats, ToCollection,
    },
    error::ModelError,
    utils::{
        json::{determine_category_by_value, JsonToModel},
        json_related_sort, name_modifier, parse_order,
    },
};

use super::{
    geofence_property::{Basic, FullPropertyModel},
    sea_orm_active_enums::Type,
    *,
};

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

#[derive(Serialize, Deserialize, FromQueryResult)]
pub struct OnlyParent {
    pub parent: Option<u32>,
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

    fn to_feature(
        self,
        property_map: &HashMap<u32, Vec<FullPropertyModel>>,
        name_map: &HashMap<u32, String>,
        args: &ApiQueryArgs,
    ) -> Result<Feature, ModelError> {
        let mut has_manual_parent = String::from("");
        let mut properties = if let Some(properties) = property_map.get(&self.id) {
            properties
                .into_iter()
                .map(|prop| {
                    if prop.name == "parent"
                        && prop.value.is_some()
                        && args.ignoremanualparent.is_none()
                    {
                        has_manual_parent = prop.value.as_ref().unwrap().clone();
                    }
                    prop.parse_db_value(&self)
                })
                .collect::<Vec<Basic>>()
        } else {
            vec![]
        };

        let parent_name = if has_manual_parent.is_empty() {
            if let Some(parent_id) = self.parent {
                if let Some(name) = name_map.get(&parent_id) {
                    Some(name.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            Some(has_manual_parent)
        };
        if args.internal.unwrap_or(false) {
            properties.push(Basic {
                name: "__id",
                value: serde_json::Value::from(self.id),
            });
            properties.push(Basic {
                name: "__name",
                value: serde_json::Value::from(self.name.to_string()),
            });
            properties.push(Basic {
                name: "__mode",
                value: serde_json::Value::from(self.mode.to_value()),
            });
            properties.push(Basic {
                name: "__parent",
                value: serde_json::Value::from(self.parent),
            });
        }
        if args.name.unwrap_or(false) {
            properties.push(Basic {
                name: "name",
                value: serde_json::Value::from(self.name),
            });
        }
        if args.id.unwrap_or(false) {
            properties.push(Basic {
                name: "id",
                value: serde_json::Value::from(self.id),
            });
        }
        if args.mode.unwrap_or(false) {
            properties.push(Basic {
                name: "mode",
                value: serde_json::Value::from(self.mode.to_value()),
            });
        }
        if args.group.unwrap_or(false) && parent_name.is_some() {
            properties.push(Basic {
                name: "group",
                value: serde_json::Value::from(parent_name.clone()),
            });
        }
        if args.parent.unwrap_or(false) {
            properties.push(Basic {
                name: "parent",
                value: serde_json::Value::from(parent_name.clone()),
            });
        }

        let mut feature = Feature {
            geometry: Some(Geometry::from_json_value(self.geometry)?),
            ..Feature::default()
        };
        for prop in properties.into_iter() {
            if prop.name.eq("parent") {
                feature.set_property(prop.name, parent_name.clone())
            } else {
                feature.set_property(prop.name, prop.value)
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
                        .map(|prop| {
                            let property_id = prop.property_id.clone();
                            let mut new_json = json!(prop.parse_db_value(&record));
                            new_json["property_id"] = property_id.into();
                            new_json
                        })
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
            Ok(record) => record.to_feature(&HashMap::new(), &HashMap::new(), args),
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

        let results = results
            .into_iter()
            .filter_map(|result| {
                result
                    .to_feature(&HashMap::new(), &HashMap::new(), args)
                    .ok()
            })
            .collect::<Vec<Feature>>();

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
        args: AdminReqParsed,
    ) -> Result<PaginateResults<Vec<Json>>, DbErr> {
        let column = Column::from_str(&args.sort_by).unwrap_or(Column::Name);

        let mut paginator = Entity::find()
            .order_by(column, parse_order(&args.order))
            .filter(Column::Name.like(format!("%{}%", args.q).as_str()));

        if let Some(parent) = args.parent {
            paginator = paginator.filter(Column::Parent.eq(parent));
        }

        if let Some(geo_type) = args.geotype {
            paginator = paginator.filter(Column::GeoType.eq(geo_type));
        }
        if let Some(mode) = args.mode {
            paginator = paginator.filter(Column::Mode.eq(mode));
        }
        if let Some(project_id) = args.project {
            paginator = paginator
                .inner_join(project::Entity)
                .filter(project::Column::Id.eq(project_id));
        }

        let paginator = paginator.paginate(db, args.per_page);

        let total = paginator.num_items_and_pages().await?;

        let results = paginator.fetch_page(args.page).await?;

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

        if args.sort_by.contains("length") {
            json_related_sort(
                &mut results,
                &args.sort_by.replace(".length", ""),
                args.order,
            );
        }

        Ok(PaginateResults {
            results,
            total: total.number_of_items,
            has_prev: total.number_of_pages == args.page + 1,
            has_next: args.page + 1 < total.number_of_pages,
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

        let model = if let Some(old_model) = old_model {
            if old_model.name.ne(name) {
                Query::update_related_route_names(db, &old_model, name.clone()).await?;
            };
            new_model.id = Set(old_model.id);
            new_model.update(db).await?
        } else {
            let model = new_model.insert(db).await?;
            let prop_name_model =
                geofence_property::Query::add_db_property(db, model.id, "name").await?;
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

    async fn upsert_feature(
        conn: &DatabaseConnection,
        feat: Feature,
        parent_map: &mut HashMap<String, UnknownId>,
    ) -> Result<Model, ModelError> {
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

        let name = if let Some(name) = feat.property("__name") {
            if let Some(name) = name.as_str() {
                new_map.insert("name", serde_json::Value::String(name.to_string()));
                name.to_string()
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

        if let Some(parent) = feat.property("__parent") {
            if let Some(parent) = parent.as_str() {
                parent_map.insert(name, UnknownId::String(parent.to_string()));
            } else if let Some(parent) = parent.as_u64() {
                parent_map.insert(name, UnknownId::Number(parent as u32));
            }
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
        let mut parent_map = HashMap::<String, UnknownId>::new();
        match area {
            GeoFormats::Feature(feat) => {
                Query::upsert_feature(conn, feat, &mut parent_map).await?;
            }
            feat => {
                let fc = match feat {
                    GeoFormats::FeatureCollection(fc) => fc,
                    geometry => geometry.to_collection(None, None),
                };
                for feat in fc.into_iter() {
                    Query::upsert_feature(conn, feat, &mut parent_map).await?;
                }
            }
        };
        if !parent_map.is_empty() {
            // ensures it exists before running the try_join_all
            property::Query::get_or_create_db_prop(conn, "parent").await?;
            future::try_join_all(
                parent_map
                    .into_iter()
                    .map(|(name, parent)| Query::associate_parent(conn, name, parent)),
            )
            .await?;
        }

        Ok(())
    }

    async fn associate_parent(
        db: &DatabaseConnection,
        name: String,
        parent: UnknownId,
    ) -> Result<(), ModelError> {
        let model = Query::get_one(db, name).await?;
        let parent_model = Query::get_one(db, parent.to_string()).await?;
        let mut new_model: ActiveModel = model.into();
        new_model.parent = Set(Some(parent_model.id));
        let model = new_model.update(db).await?;
        geofence_property::Query::add_db_property(db, model.id, "parent").await?;
        Ok(())
    }

    /// Returns all geofence models, as models, that are related to the specified project
    pub async fn by_project(
        db: &DatabaseConnection,
        project_name: String,
    ) -> Result<Vec<Json>, DbErr> {
        Entity::find()
            .order_by(Column::Name, Order::Asc)
            .left_join(project::Entity)
            .filter(match project_name.parse::<u32>() {
                Ok(id) => project::Column::Id.eq(id),
                Err(_) => project::Column::Name.eq(project_name),
            })
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .column(Column::Mode)
            .column(Column::Parent)
            .into_json()
            .all(db)
            .await
    }

    /// Returns all geofence models, as features, that are related to the specified project
    pub async fn project_as_feature(
        db: &DatabaseConnection,
        project_name: String,
        args: &ApiQueryArgs,
    ) -> Result<Vec<Feature>, ModelError> {
        let time = Instant::now();

        let items = Entity::find()
            .order_by(Column::Name, Order::Asc)
            .filter(match project_name.parse::<u32>() {
                Ok(id) => project::Column::Id.eq(id),
                Err(_) => project::Column::Name.eq(project_name),
            })
            .left_join(project::Entity)
            .all(db)
            .await?;

        let mut property_map = HashMap::<u32, Vec<FullPropertyModel>>::new();

        let mut ids = vec![];

        items.iter().for_each(|item| {
            ids.push(item.id);
            if let Some(parent_id) = item.parent {
                ids.push(parent_id);
            }
        });

        let mut name_map: HashMap<u32, String> = Entity::find()
            .filter(Column::Id.is_in(ids.clone()))
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .into_model::<NameId>()
            .all(db)
            .await?
            .into_iter()
            .map(|model| (model.id, model.name))
            .collect();

        geofence_property::Entity::find()
            .filter(geofence_property::Column::GeofenceId.is_in(ids))
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
            .into_model::<FullPropertyModel>()
            .all(db)
            .await?
            .into_iter()
            .for_each(|prop| {
                if prop.name == "name" {
                    if let Some(manual_name) = prop.value.as_ref() {
                        name_map.insert(prop.geofence_id, manual_name.clone());
                    }
                }
                property_map
                    .entry(prop.geofence_id)
                    .or_insert_with(Vec::new)
                    .push(prop);
            });

        log::debug!("db query took {:?}", time.elapsed());

        let time = Instant::now();
        let items = items
            .into_iter()
            .filter_map(|result| result.to_feature(&property_map, &name_map, args).ok())
            .collect();

        log::debug!("feature conversion took {:?}", time.elapsed());
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
                    Column::Parent => match payload.as_u64() {
                        Some(id) => {
                            if id == 0 {
                                model.parent = Set(None);
                            } else {
                                model.parent = Set(Some(id as u32));
                            }
                        }
                        None => {
                            return Err(ModelError::Geofence(
                                "No valid parent_id found".to_string(),
                            ))
                        }
                    },
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

    pub async fn by_parent(
        db: &DatabaseConnection,
        parent: &UnknownId,
    ) -> Result<FeatureCollection, ModelError> {
        let parent_id = match parent {
            UnknownId::Number(id) => id.clone(),
            UnknownId::String(name) => {
                let parent = Entity::find().filter(Column::Name.eq(name)).one(db).await?;
                if let Some(parent) = parent {
                    parent.id
                } else {
                    return Err(ModelError::Geofence("Parent not found".to_string()));
                }
            }
        };
        let items = Entity::find()
            .filter(Column::Parent.eq(parent_id))
            .all(db)
            .await?;
        let args = ApiQueryArgs::default();

        let items: Vec<Feature> = items
            .into_iter()
            .filter_map(|result| {
                result
                    .to_feature(&HashMap::new(), &HashMap::new(), &args)
                    .ok()
            })
            .collect();

        Ok(items.to_collection(None, None))
    }

    pub async fn unique_parents(db: &DatabaseConnection) -> Result<Vec<Json>, ModelError> {
        let items = Entity::find()
            .filter(Column::Parent.is_not_null())
            .select_only()
            .column(Column::Parent)
            .distinct()
            .into_model::<OnlyParent>()
            .all(db)
            .await?;
        let items = Entity::find()
            .order_by(Column::Name, Order::Asc)
            .filter(Column::Id.is_in(items.into_iter().filter_map(|item| {
                if let Some(parent) = item.parent {
                    Some(parent)
                } else {
                    None
                }
            })))
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .distinct()
            .into_json()
            .all(db)
            .await?;
        Ok(items)
    }
}
