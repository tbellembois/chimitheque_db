use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use chimitheque_types::{
    entity::Entity, person::Person, requestfilter::RequestFilter, storage::Storage,
    storelocation::StoreLocation,
};
use log::debug;
use rusqlite::Connection;
use sea_query::{Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;

use crate::{
    entity::get_entities, permission::Permission, person::get_people, product::get_products,
    storage::get_storages, storelocation::get_store_locations,
};

#[derive(Debug, PartialEq, Eq)]
pub enum CasbinError {
    MissingPerson,
    MissingEntity,
    MissingEntityId,
    MissingStoreLocationId,
    MissingStoreLocation,
    MissingStorage,
    MissingStorageId,
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

/// Helper function to check if two sets of u64 have common values.
fn has_common_elements(set1: &HashSet<u64>, set2: &HashSet<u64>) -> bool {
    for &value in set1 {
        if set2.contains(&value) {
            return true;
        }
    }
    false
}

/// Helper function to get the entity IDs of a person by ID.
fn get_person_entities_ids(
    db_connection: &Connection,
    person_id: u64,
) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
    // Get the person from the database.
    let person = get_person_by_id(db_connection, person_id)?;

    // Get the person entities.
    let person_entities: Vec<Entity> = person.entities.clone().unwrap_or_default();

    // Check that the entities have valid entity_id.
    if person_entities.iter().any(|e| e.entity_id.is_none()) {
        return Err(Box::new(CasbinError::MissingEntityId));
    }

    // Get the entity IDs of the person.
    // We can unwrap safely because we checked for missing entity_id above.
    let person_entities_ids: Vec<u64> = person_entities
        .iter()
        .map(|e| e.entity_id.unwrap())
        .collect();

    Ok(person_entities_ids)
}

/// Helper function to get a person by ID.
fn get_person_by_id(
    db_connection: &Connection,
    person_id: u64,
) -> Result<Person, Box<dyn std::error::Error + Send + Sync>> {
    let (people, nb_results) = get_people(
        db_connection,
        RequestFilter {
            id: Some(person_id),
            ..Default::default()
        },
        1,
    )?;

    if nb_results == 0 {
        return Err(Box::new(CasbinError::MissingPerson));
    }

    let person = people.first().ok_or(CasbinError::MissingPerson)?;
    Ok((*person).clone())
}

/// Helper function to get a store location by ID.
fn get_store_location_by_id(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<StoreLocation, Box<dyn std::error::Error + Send + Sync>> {
    let (store_locations, nb_results) = get_store_locations(
        db_connection,
        RequestFilter {
            id: Some(store_location_id),
            ..Default::default()
        },
        1,
    )?;

    if nb_results == 0 {
        return Err(Box::new(CasbinError::MissingStoreLocation));
    }

    let store_location = store_locations
        .first()
        .ok_or(CasbinError::MissingStoreLocation)?;
    Ok((*store_location).clone())
}

/// Helper function to get a storage by ID.
fn get_storage_by_id(
    db_connection: &Connection,
    storage_id: u64,
) -> Result<Storage, Box<dyn std::error::Error + Send + Sync>> {
    let (storages, nb_results) = get_storages(
        db_connection,
        RequestFilter {
            id: Some(storage_id),
            ..Default::default()
        },
        1,
    )?;

    if nb_results == 0 {
        return Err(Box::new(CasbinError::MissingStorage));
    }

    let storage = storages.first().ok_or(CasbinError::MissingStorage)?;
    Ok((*storage).clone())
}

/// Helper function to get a entity by ID.
fn get_entity_by_id(
    db_connection: &Connection,
    entity_id: u64,
) -> Result<Entity, Box<dyn std::error::Error + Send + Sync>> {
    let (entities, nb_results) = get_entities(
        db_connection,
        RequestFilter {
            id: Some(entity_id),
            ..Default::default()
        },
        1,
    )?;

    if nb_results == 0 {
        return Err(Box::new(CasbinError::MissingEntity));
    }

    let entity = entities.first().ok_or(CasbinError::MissingEntity)?;
    Ok((*entity).clone())
}

/// Returns a string policy for the casbin adapter.
/// https://docs.rs/casbin/latest/casbin/struct.StringAdapter.html
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

    debug!("select_sql: {}", sql_query.clone().as_str());
    debug!("select_values: {:?}", sql_values);

    let mut stmt = db_connection.prepare(sql_query.as_str())?;
    let mut rows = stmt.query(&*sql_values.as_params())?;

    while let Some(row) = rows.next()? {
        result += format!(
            "p, {}, {}, {}, {}",
            row.get_unwrap::<_, u64>("person"),
            row.get_unwrap::<_, String>("permission_name"),
            row.get_unwrap::<_, String>("permission_item"),
            row.get_unwrap::<_, i64>("permission_entity")
        )
        .as_str();
    }

    Ok(result)
}

/// Checks if a person is associated with an entity.
pub fn match_person_is_in_entity(
    db_connection: &Connection,
    person_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_person_is_in_entity: {} {}", person_id, entity_id);

    // Get person.
    let person = get_person_by_id(db_connection, person_id)?;

    // Return true on orphans.
    if person.entities.is_none() {
        return Ok(true);
    }

    // Return true on admins.
    if person.is_admin {
        return Ok(true);
    }

    // Get the person entities ids.
    let person_entities_ids = get_person_entities_ids(db_connection, person_id)?;

    // Check if the entity_id is in the person entities ids.
    let result = person_entities_ids.contains(&entity_id);

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if two persons are associated with the same entity.
pub fn match_person_is_in_person_entity(
    db_connection: &Connection,
    person_id: u64,
    other_person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "match_person_is_in_person_entity: {} {}",
        person_id, other_person_id
    );

    // Get person.
    let person = get_person_by_id(db_connection, person_id)?;

    if person.is_admin {
        return Ok(true);
    }

    // Get the person and other person the entities ids.
    let person_entities_ids = get_person_entities_ids(db_connection, person_id)?;
    let other_person_entities_ids = get_person_entities_ids(db_connection, other_person_id)?;

    // Building sets for the vec_of_u64_have_common_values function.
    let set_person_entities_ids: HashSet<u64> = person_entities_ids.into_iter().collect();
    let set_other_person_entities_ids: HashSet<u64> =
        other_person_entities_ids.into_iter().collect();

    let result = has_common_elements(&set_person_entities_ids, &set_other_person_entities_ids);

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a person is associated with the entity of the store location.
pub fn match_person_is_in_store_location_entity(
    db_connection: &Connection,
    person_id: u64,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "match_person_is_in_store_location_entity: {} {}",
        person_id, store_location_id
    );

    // Get person.
    let person = get_person_by_id(db_connection, person_id)?;

    if person.is_admin {
        return Ok(true);
    }

    // Get the person entities ids.
    let person_entities_ids = get_person_entities_ids(db_connection, person_id)?;

    // Get the store location from the database.
    let store_location = get_store_location_by_id(db_connection, store_location_id)?;

    // Then its entity.
    let Some(entity) = store_location.entity else {
        return Err(Box::new(CasbinError::MissingEntity));
    };

    // Then its entity id.
    let Some(entity_id) = entity.entity_id else {
        return Err(Box::new(CasbinError::MissingEntityId));
    };

    // Check if the entity_id is in the person entities ids.
    let result = person_entities_ids.contains(&entity_id);

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a person is associated with the entity of the storage.
pub fn match_person_is_in_storage_entity(
    db_connection: &Connection,
    person_id: u64,
    storage_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "match_person_is_in_storage_entity: {} {}",
        person_id, storage_id
    );

    // Get person.
    let person = get_person_by_id(db_connection, person_id)?;

    if person.is_admin {
        return Ok(true);
    }

    // Get the person entities ids.
    let person_entities_ids = get_person_entities_ids(db_connection, person_id)?;

    // Get the storage from the database.
    let storage = get_storage_by_id(db_connection, storage_id)?;

    // Then its store location.
    let store_location = storage.store_location;

    // Then its entity.
    let Some(entity) = store_location.entity else {
        return Err(Box::new(CasbinError::MissingEntity));
    };

    // Then its entity id.
    let Some(entity_id) = entity.entity_id else {
        return Err(Box::new(CasbinError::MissingEntityId));
    };

    // Check if the storage_id is in the person entities ids.
    let result = person_entities_ids.contains(&entity_id);

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a product has storages.
pub fn match_product_has_storages(
    db_connection: &Connection,
    product_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_product_has_storages: {}", product_id);

    // Get the product from the database.
    let (products, nb_results) = get_products(
        db_connection,
        RequestFilter {
            id: Some(product_id),
            ..Default::default()
        },
        1,
    )?;

    if nb_results == 0 {
        return Err(Box::new(CasbinError::MissingProduct));
    }

    let product = products.first().ok_or(CasbinError::MissingProduct)?;
    let nb_storages = product.product_tsc.unwrap_or_default();

    let result = nb_storages > 0;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a store location has children.
pub fn match_store_location_has_children(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_store_location_has_children: {}", store_location_id);

    // Get the number of store location children.
    let (_, nb_results) = get_store_locations(
        db_connection,
        RequestFilter {
            store_location: Some(store_location_id),
            ..Default::default()
        },
        1,
    )?;

    let result = nb_results > 0;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a store location has storages.
pub fn match_store_location_has_storages(
    db_connection: &Connection,
    store_location_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_store_location_has_storages: {}", store_location_id);

    // Get the store location from the database.
    let store_location = get_store_location_by_id(db_connection, store_location_id)?;

    let nb_storages = store_location
        .store_location_nb_storages
        .unwrap_or_default();

    let result = nb_storages > 0;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a person is an admin.
pub fn match_person_is_admin(
    db_connection: &Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_person_is_admin: {}", person_id);

    // Get the person from the database.
    let person = get_person_by_id(db_connection, person_id)?;

    let result = person.is_admin;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a person is a manager.
pub fn match_person_is_manager(
    db_connection: &Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_person_is_manager: {}", person_id);

    // Get the person from the database.
    let person = get_person_by_id(db_connection, person_id)?;

    let managed_entities = person.managed_entities.unwrap_or_default();

    let result = !managed_entities.is_empty();

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if an entity has members.
pub fn match_entity_has_members(
    db_connection: &Connection,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_entity_has_members: {}", entity_id);

    // Get the entity from the database.
    let entity = get_entity_by_id(db_connection, entity_id)?;

    let nb_members = entity.entity_nb_people.unwrap_or_default();

    let result = nb_members > 0;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if an entity has store locations.
pub fn match_entity_has_store_locations(
    db_connection: &Connection,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_entity_has_store_locations: {}", entity_id);

    // Get the entity from the database.
    let entity = get_entity_by_id(db_connection, entity_id)?;

    let nb_store_locations = entity.entity_nb_store_locations.unwrap_or_default();

    let result = nb_store_locations > 0;

    debug!("result: {}", result);

    Ok(result)
}

/// Checks if a storage is in an entity.
pub fn match_storage_is_in_entity(
    db_connection: &Connection,
    storage_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("match_storage_is_in_entity: {} {}", storage_id, entity_id);

    // Get the storage from the database.
    let storage = get_storage_by_id(db_connection, storage_id)?;

    if let Some(store_location_id) = storage.store_location.store_location_id {
        let store_location = get_store_location_by_id(db_connection, store_location_id)?;

        if let Some(entity) = store_location.entity {
            if let Some(this_entity_id) = entity.entity_id {
                let result = this_entity_id == entity_id;

                debug!("result: {}", result);

                Ok(result)
            } else {
                Err(Box::new(CasbinError::MissingEntityId))
            }
        } else {
            Err(Box::new(CasbinError::MissingEntity))
        }
    } else {
        Err(Box::new(CasbinError::MissingStoreLocationId))
    }
}

/// Checks if a store location is in an entity.
pub fn match_store_location_is_in_entity(
    db_connection: &Connection,
    store_location_id: u64,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "match_store_location_is_in_entity: {} {}",
        store_location_id, entity_id
    );

    // Get store location from the database.
    let store_location = get_store_location_by_id(db_connection, store_location_id)?;

    if let Some(entity) = store_location.entity {
        if let Some(this_entity_id) = entity.entity_id {
            let result = this_entity_id == entity_id;

            debug!("result: {}", result);

            Ok(result)
        } else {
            Err(Box::new(CasbinError::MissingEntityId))
        }
    } else {
        Err(Box::new(CasbinError::MissingEntity))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::{connect_test, init_db, insert_fake_values};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = connect_test();
        init_db(&mut db_connection).unwrap();
        insert_fake_values(&mut db_connection).unwrap();
        db_connection
    }

    #[test]
    fn test_casbin() {
        init_logger();

        let db_connection = init_test_db();

        assert!(match_person_is_in_entity(&db_connection, 2, 1).unwrap());
        assert!(!match_person_is_in_entity(&db_connection, 2, 2).unwrap());

        assert!(match_person_is_in_person_entity(&db_connection, 2, 4).unwrap());
        assert!(!match_person_is_in_person_entity(&db_connection, 2, 3).unwrap());

        assert!(match_person_is_in_store_location_entity(&db_connection, 2, 1).unwrap());
        assert!(!match_person_is_in_store_location_entity(&db_connection, 2, 3).unwrap());

        assert!(match_person_is_in_storage_entity(&db_connection, 2, 1).unwrap());
        assert!(!match_person_is_in_storage_entity(&db_connection, 2, 3).unwrap());

        assert!(match_product_has_storages(&db_connection, 1).unwrap());
        assert!(match_product_has_storages(&db_connection, 2).unwrap());
        assert!(!match_product_has_storages(&db_connection, 5).unwrap());

        assert!(match_store_location_has_children(&db_connection, 4).unwrap());
        assert!(!match_store_location_has_children(&db_connection, 1).unwrap());

        assert!(match_store_location_has_storages(&db_connection, 4).unwrap());
        assert!(!match_store_location_has_storages(&db_connection, 5).unwrap());

        assert!(match_person_is_admin(&db_connection, 1).unwrap());
        assert!(!match_person_is_admin(&db_connection, 2).unwrap());

        assert!(match_person_is_manager(&db_connection, 2).unwrap());
        assert!(!match_person_is_manager(&db_connection, 4).unwrap());

        assert!(match_entity_has_members(&db_connection, 2).unwrap());
        assert!(!match_entity_has_members(&db_connection, 3).unwrap());

        assert!(match_entity_has_store_locations(&db_connection, 2).unwrap());
        assert!(!match_entity_has_store_locations(&db_connection, 3).unwrap());

        assert!(match_store_location_is_in_entity(&db_connection, 1, 1).unwrap());
        assert!(!match_store_location_is_in_entity(&db_connection, 1, 2).unwrap());

        assert!(match_storage_is_in_entity(&db_connection, 1, 1).unwrap());
        assert!(!match_storage_is_in_entity(&db_connection, 1, 3).unwrap());
    }
}
