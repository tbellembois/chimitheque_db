use chimitheque_types::error::ParseError;
use chimitheque_types::permission::Permission as PermissionStruct;
use chimitheque_types::permission::PermissionItem;
use chimitheque_types::permission::PermissionName;
use chimitheque_types::person::Person;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;
use std::str::FromStr;

/// Table and column names of the `permission` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Permission {
    /// The table.
    Table,
    /// The `person` column.
    Person,
    /// The `permission_entity` column.
    PermissionEntity,
    /// The `permission_name` column.
    PermissionName,
    /// The `permission_item` column.
    PermissionItem,
}

/// A permission read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct PermissionWrapper(pub PermissionStruct);

impl TryFrom<&Row<'_>> for PermissionWrapper {
    type Error = ParseError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let permission_item_string: String = row.get_unwrap("permission_item");
        let permission_item: PermissionItem = PermissionItem::from_str(&permission_item_string)?;

        let permission_name_string: String = row.get_unwrap("permission_name");
        let permission_name: PermissionName = PermissionName::from_str(&permission_name_string)?;

        let permission_entity: Option<u64> = match row.get("permission_entity") {
            Ok(Some(permission_entity)) => Some(permission_entity),
            _ => None,
        };

        Ok(Self(PermissionStruct {
            permission_item,
            permission_name,
            permission_entity,
            person: Person {
                person_id: row.get_unwrap("person_id"),
                person_email: row.get_unwrap("person_email"),
                ..Default::default()
            },
        }))
    }
}

/// Returns whether the person can write the `item` (e.g. `storages` or
/// `entities`) of the entity: a `w` or `all` permission on the item or on all
/// items, for the entity or for all entities.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn person_can_write_in_entity(
    db_connection: &rusqlite::Connection,
    person_id: u64,
    item: &PermissionItem,
    entity_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM permission
           WHERE person = ?1
             AND permission_item IN (?2, 'all')
             AND permission_name IN ('w', 'all')
             AND (permission_entity = ?3 OR permission_entity IS NULL))",
        (person_id, item.to_string(), entity_id),
        |row| row.get(0),
    )?)
}

/// Returns whether the person can read the restricted products. The minimal
/// permission is `r` on `rproducts`; people who can write the products (`w` or
/// `all` on `products` or `rproducts`), managers and admins (`all` item) can
/// read them too. The entity does not matter: products do not belong to one.
///
/// # Errors
///
/// Returns an error if the query fails.
pub fn person_can_read_restricted_products(
    db_connection: &rusqlite::Connection,
    person_id: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    Ok(db_connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM permission
           WHERE person = ?1
             AND ((permission_item IN ('rproducts', 'all') AND permission_name IN ('r', 'w', 'all'))
               OR (permission_item = 'products' AND permission_name IN ('w', 'all'))))",
        [person_id],
        |row| row.get(0),
    )?)
}

#[cfg(test)]
#[path = "permission_tests.rs"]
mod permission_tests;
