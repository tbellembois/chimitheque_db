use chimitheque_types::error::ParseError;
use chimitheque_types::permission::Permission as PermissionStruct;
use chimitheque_types::permission::PermissionItem;
use chimitheque_types::permission::PermissionName;
use chimitheque_types::person::Person;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;
use std::str::FromStr;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Permission {
    Table,
    PermissionId,
    Person,
    PermissionEntity,
    PermissionName,
    PermissionItem,
}

#[derive(Debug, Serialize, Default)]
pub struct PermissionWrapper(pub PermissionStruct);

impl TryFrom<&Row<'_>> for PermissionWrapper {
    type Error = ParseError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let permission_item_string: String = row.get_unwrap("permission_item");
        let permission_item: PermissionItem = PermissionItem::from_str(&permission_item_string)?;

        let permission_name_string: String = row.get_unwrap("permission_name");
        let permission_name: PermissionName = PermissionName::from_str(&permission_name_string)?;

        let permission_entity: i64 = row.get_unwrap("permission_entity");

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
