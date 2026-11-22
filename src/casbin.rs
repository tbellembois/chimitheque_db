//! The matcher functions of the casbin policy, in
//! `chimitheque_back/src/casbin/policy.conf`. Each one runs a simple query.

use std::fmt::{Display, Formatter};

use log::debug;
use rusqlite::Connection;
use sea_query::{Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;

use crate::{
    entity::{entity_has_members, entity_has_store_locations},
    permission::Permission,
    person::{is_person_admin, managed_entity_ids, member_entity_ids},
    product::product_has_storages,
    storage::storage_entity_id,
    storelocation::{
        store_location_entity_id, store_location_has_children, store_location_has_storages,
    },
};

/// Why a casbin matcher function can not answer.
#[derive(Debug, PartialEq, Eq)]
pub enum CasbinError {
    /// No person found for the id.
    MissingPerson,
    /// No entity found for the id.
    MissingEntity,
    /// An entity has no id.
    MissingEntityId,
    /// A store location has no id.
    MissingStoreLocationId,
    /// No store location found for the id.
    MissingStoreLocation,
    /// No storage found for the id.
    MissingStorage,
    /// A storage has no id.
    MissingStorageId,
    /// No product found for the id.
    MissingProduct,
}

impl Display for CasbinError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CasbinError::MissingPerson => write!(f, "casbin: missing person"),
            CasbinError::MissingEntity => write!(f, "casbin: missing entity"),
            CasbinError::MissingEntityId => write!(f, "casbin: missing entity id"),
            CasbinError::MissingStoreLocationId => write!(f, "casbin: missing store location id"),
            CasbinError::MissingStoreLocation => write!(f, "casbin: missing store location"),
            CasbinError::MissingStorage => write!(f, "casbin: missing storage"),
            CasbinError::MissingStorageId => write!(f, "casbin: missing storage id"),
            CasbinError::MissingProduct => write!(f, "casbin: missing product"),
        }
    }
}

impl std::error::Error for CasbinError {}

/// Returns a string policy for the casbin adapter.
/// <https://docs.rs/casbin/latest/casbin/struct.StringAdapter.html>
pub fn to_string_adapter(
    db_connection: &Connection,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut result = String::new();

    let (sql_query, sql_values) = Query::select()
        .from(Permission::Table)
        .columns([
            Permission::Person,
            Permission::PermissionName,
            Permission::PermissionItem,
            Permission::PermissionEntity,
        ])
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", sql_query.as_str());
    debug!("select_values: {sql_values:?}");

    let mut stmt = db_connection.prepare(sql_query.as_str())?;
    let mut rows = stmt.query(&*sql_values.as_params())?;

    while let Some(row) = rows.next()? {
        let mut permission_entity_string = String::new();
        if let Ok(maybe_permission_entity) = row.get::<_, Option<u64>>("permission_entity")
            && let Some(permission_entity) = maybe_permission_entity
        {
            permission_entity_string = permission_entity.to_string();
        }

        result += format!(
            "p, {}, {}, {}, {}\n",
            row.get_unwrap::<_, u64>("person"),
            row.get_unwrap::<_, String>("permission_name"),
            row.get_unwrap::<_, String>("permission_item"),
            permission_entity_string
        )
        .as_str();
    }

    debug!("result: {result}");

    Ok(result)
}

/// Checks if a person is a member of an entity. An admin, or a person with no
/// entity, is a member of all entities.
///
/// # Errors
///
/// Returns `CasbinError::MissingPerson` for an unknown person: they would
/// otherwise have no entity.
pub fn match_person_is_in_entity(
    db_connection: &Connection,
    person_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_person_is_in_entity: {person_id} {entity_id}");

    let person_exists: bool = db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM person WHERE person_id = ?1)",
        [person_id],
        |row| row.get(0),
    )?;
    if !person_exists {
        return Err(Box::new(CasbinError::MissingPerson));
    }

    if is_person_admin(db_connection, person_id)? {
        return Ok(true);
    }

    let member_entity_ids = member_entity_ids(db_connection, person_id)?;
    Ok(member_entity_ids.is_empty() || member_entity_ids.contains(&entity_id))
}

/// Checks if a product has storages, archived and history ones included.
pub fn match_product_has_storages(
    db_connection: &Connection,
    product_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    product_has_storages(db_connection, product_id)
}

/// Checks if a store location has children.
pub fn match_store_location_has_children(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    store_location_has_children(db_connection, store_location_id)
}

/// Checks if a store location has storages.
pub fn match_store_location_has_storages(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    store_location_has_storages(db_connection, store_location_id)
}

/// Checks if a person is an admin.
pub fn match_person_is_admin(
    db_connection: &Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    is_person_admin(db_connection, person_id)
}

/// Checks if a person manages an entity.
pub fn match_person_is_manager(
    db_connection: &Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(!managed_entity_ids(db_connection, person_id)?.is_empty())
}

/// Checks if an entity has members.
pub fn match_entity_has_members(
    db_connection: &Connection,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    entity_has_members(db_connection, entity_id)
}

/// Checks if an entity has store locations.
pub fn match_entity_has_store_locations(
    db_connection: &Connection,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    entity_has_store_locations(db_connection, entity_id)
}

/// Checks if a storage is in an entity; false for an unknown storage.
pub fn match_storage_is_in_entity(
    db_connection: &Connection,
    storage_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(storage_entity_id(db_connection, storage_id)? == Some(entity_id))
}

/// Checks if a store location is in an entity; false for an unknown store
/// location.
pub fn match_store_location_is_in_entity(
    db_connection: &Connection,
    store_location_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(store_location_entity_id(db_connection, store_location_id)? == Some(entity_id))
}

#[cfg(test)]
#[path = "casbin_tests.rs"]
mod casbin_tests;
