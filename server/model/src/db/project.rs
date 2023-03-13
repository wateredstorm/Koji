//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use crate::utils::{json::JsonToModel, json_related_sort, parse_order};

use super::*;
use sea_orm::entity::prelude::*;
use serde_json::json;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    pub api_endpoint: Option<String>,
    pub api_key: Option<String>,
    pub scanner: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::geofence::Entity")]
    Geofence,
}

impl Related<geofence::Entity> for Entity {
    fn to() -> RelationDef {
        geofence_project::Relation::Geofence.def()
    }
    fn via() -> Option<RelationDef> {
        Some(geofence_project::Relation::Project.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    fn get_related_geofences(&self) -> Select<geofence::Entity> {
        self.find_related(geofence::Entity)
            .select_only()
            .column(geofence::Column::Id)
            .column(geofence::Column::Name)
    }
}

pub struct Query;

impl Query {
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
                json.insert(
                    "geofences".to_string(),
                    json!(record
                        .get_related_geofences()
                        .into_model::<NameId>()
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|p| p.id)
                        .collect::<Vec<u32>>()),
                );
                Ok(json!(json))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn paginate(
        db: &DatabaseConnection,
        page: u64,
        posts_per_page: u64,
        order: String,
        sort_by: String,
        q: String,
    ) -> Result<PaginateResults<Vec<Json>>, DbErr> {
        let paginator = project::Entity::find()
            .order_by(
                Column::from_str(&sort_by).unwrap_or(Column::Name),
                parse_order(&order),
            )
            .filter(Column::Name.like(format!("%{}%", q).as_str()))
            .paginate(db, posts_per_page);
        let total = paginator.num_items_and_pages().await?;

        let results: Vec<Model> = match paginator.fetch_page(page).await {
            Ok(results) => results,
            Err(err) => {
                println!("[project] Error paginating, {:?}", err);
                vec![]
            }
        };

        let geofences = future::try_join_all(
            results
                .iter()
                .map(|result| result.get_related_geofences().into_json().all(db)),
        )
        .await?;

        let mut results: Vec<Json> = results
            .into_iter()
            .enumerate()
            .map(|(i, project)| {
                json!({
                    "id": project.id,
                    "name": project.name,
                    "api_endpoint": project.api_endpoint,
                    "api_key": project.api_key,
                    "scanner": project.scanner,
                    // "created_at": fence.created_at,
                    // "updated_at": fence.updated_at,
                    "geofences": geofences[i],
                })
            })
            .collect();

        if sort_by == "geofences" {
            json_related_sort(&mut results, &sort_by, order);
        }

        Ok(PaginateResults {
            results,
            total: total.number_of_items,
            has_prev: total.number_of_pages == page + 1,
            has_next: page + 1 < total.number_of_pages,
        })
    }

    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        project::Entity::find().all(db).await
    }

    pub async fn get_json_cache(db: &DatabaseConnection) -> Result<Vec<sea_orm::JsonValue>, DbErr> {
        let results = Entity::find()
            .order_by(Column::Name, Order::Asc)
            .all(db)
            .await?;
        let geofences = future::try_join_all(results.iter().map(|result| {
            result
                .get_related_geofences()
                .into_model::<NameId>()
                .all(db)
        }))
        .await?;

        Ok(results
            .into_iter()
            .enumerate()
            .map(|(i, model)| {
                json!({
                    "id": model.id,
                    "name": model.name,
                    "geofences": geofences[i].iter().map(|r| r.id).collect::<Vec<u32>>()
                })
            })
            .collect())
    }

    pub async fn get_scanner_project(db: &DatabaseConnection) -> Result<Option<Model>, DbErr> {
        project::Entity::find()
            .filter(Column::Scanner.eq(true))
            .filter(Column::ApiEndpoint.is_not_null())
            .one(db)
            .await
    }

    pub async fn upsert_related_geofences(
        db: &DatabaseConnection,
        json: &serde_json::Value,
        geofence_id: u32,
    ) -> Result<(), DbErr> {
        if let Some(geofences) = json.get("geofences") {
            if let Some(geofences) = geofences.as_array() {
                geofence_project::Query::upsert_related_by_project_id(db, geofences, geofence_id)
                    .await?;
            };
        };
        Ok(())
    }

    pub async fn upsert(db: &DatabaseConnection, id: u32, json: Json) -> Result<Model, ModelError> {
        let old_model: Option<Model> = Entity::find_by_id(id).one(db).await?;
        let mut new_model = json.to_project()?;

        let model = if let Some(old_model) = old_model {
            new_model.id = Set(old_model.id);
            new_model.update(db).await?
        } else {
            new_model.insert(db).await?
        };
        Query::upsert_related_geofences(db, &json, model.id).await?;

        Ok(model)
    }

    pub async fn upsert_json_return(
        db: &DatabaseConnection,
        id: u32,
        json: Json,
    ) -> Result<Json, ModelError> {
        let result = Query::upsert(db, id, json).await?;
        Ok(json!(result))
    }

    pub async fn delete(db: &DatabaseConnection, id: u32) -> Result<DeleteResult, DbErr> {
        let record = Entity::delete_by_id(id).exec(db).await?;
        Ok(record)
    }

    pub async fn search(db: &DatabaseConnection, search: String) -> Result<Vec<Json>, DbErr> {
        Ok(Entity::find()
            .filter(Column::Name.like(format!("%{}%", search).as_str()))
            .into_json()
            .all(db)
            .await?)
    }
}
