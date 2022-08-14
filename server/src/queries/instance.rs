use super::*;
use crate::db::{schema::instance::dsl::*, sql_types::InstanceType::auto_quest};
use crate::models::scanner::Instance;

pub fn query_all_instances(conn: &MysqlConnection) -> Result<Vec<Instance>, DbError> {
    let items = instance
        .load::<Instance>(conn)
        .expect("Error loading instances");
    Ok(items)
}

pub fn query_quest_instances(conn: &MysqlConnection) -> Result<Vec<Instance>, DbError> {
    let items = instance
        .filter(type_.eq(auto_quest))
        .load::<Instance>(conn)
        .expect("Error loading instances");
    Ok(items)
}

pub fn query_instance_route(
    conn: &MysqlConnection,
    instance_name: &String,
) -> Result<Instance, DbError> {
    let items = instance
        .filter(name.eq(instance_name))
        .first::<Instance>(conn)
        .expect("No instance found");
    Ok(items)
}