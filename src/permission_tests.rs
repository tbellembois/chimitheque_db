#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::permission::{person_can_read_restricted_products, person_can_write_in_entity};
    use crate::test_utils::profiles::*;
    use chimitheque_types::permission::PermissionItem;
    use rusqlite::Connection;

    // The fixture, where new.user (person 6) has only the given permissions,
    // as (name, item, entity) tuples.
    const PERSON_ID: u64 = 6;

    fn init_test_permissions(permissions: &[(&str, &str, Option<u64>)]) -> Connection {
        let db = crate::test_utils::init_test_with_fixture();
        db.execute("DELETE FROM permission WHERE person = ?1", [PERSON_ID])
            .unwrap();
        for (name, item, entity) in permissions {
            db.execute(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (?1, ?2, ?3, ?4)",
                (PERSON_ID, name, item, entity),
            )
            .unwrap();
        }
        db
    }

    fn can_write_storages(db: &Connection, entity_id: u64) -> bool {
        person_can_write_in_entity(db, PERSON_ID, &PermissionItem::Storages, entity_id).unwrap()
    }

    #[test]
    fn test_person_can_write_in_entity() {
        // On the entity only.
        let db = init_test_permissions(&[("w", "storages", Some(1))]);
        assert!(can_write_storages(&db, 1));
        assert!(!can_write_storages(&db, 2));

        // On all items of the entity, e.g. a manager.
        let db = init_test_permissions(&[("all", "all", Some(1))]);
        assert!(can_write_storages(&db, 1));
        assert!(!can_write_storages(&db, 2));

        // On all entities, e.g. an admin.
        let db = init_test_permissions(&[("all", "all", None)]);
        assert!(can_write_storages(&db, 1));
        assert!(can_write_storages(&db, 2));

        // Read only, or another item.
        let db = init_test_permissions(&[
            ("r", "storages", None),
            ("w", "products", None),
            ("w", "entities", Some(1)),
        ]);
        assert!(!can_write_storages(&db, 1));
        assert!(person_can_write_in_entity(&db, PERSON_ID, &PermissionItem::Entities, 1).unwrap());
    }

    #[test]
    fn test_person_can_read_restricted_products() {
        let can_read = |permissions: &[(&str, &str, Option<u64>)]| {
            let db = init_test_permissions(permissions);
            person_can_read_restricted_products(&db, PERSON_ID).unwrap()
        };

        // The minimal permission, writing the (restricted) products, managers
        // and admins.
        assert!(can_read(&[("r", "rproducts", None)]));
        assert!(can_read(&[("w", "products", None)]));
        assert!(can_read(&[("w", "rproducts", None)]));
        assert!(can_read(&[("all", "all", Some(2))]));
        assert!(can_read(&[("all", "all", None)]));
        assert!(can_read(&[("r", "all", None)]));

        // Reading the products, other items, no or unused permission names.
        assert!(!can_read(&[]));
        assert!(!can_read(&[("r", "products", None)]));
        assert!(!can_read(&[("w", "storages", Some(1))]));
        assert!(!can_read(&[("n", "rproducts", None)]));
        assert!(!can_read(&[("b", "rproducts", None)]));
    }

    #[test]
    fn test_profiles_can_write_in_entity() {
        let db = crate::test_utils::init_test_with_fixture();
        let can_write = |person_id, item: &PermissionItem, entity_id| {
            person_can_write_in_entity(&db, person_id, item, entity_id).unwrap()
        };

        for (person_id, chemistry_storages, biology_storages, chemistry_entity) in [
            (ADMIN, true, true, true),
            (CHEMISTRY_MANAGER, true, false, true),
            (CHEMISTRY_USER, true, false, false),
            (BIOLOGY_MANAGER, false, true, false),
            // Reads only.
            (BIOLOGY_USER, false, false, false),
            (NEW_USER, false, false, false),
            (ALL_READER, false, false, false),
            (PRODUCT_EDITOR, false, false, false),
        ] {
            assert_eq!(
                can_write(person_id, &PermissionItem::Storages, 1),
                chemistry_storages,
                "{person_id}"
            );
            assert_eq!(
                can_write(person_id, &PermissionItem::Storages, 2),
                biology_storages,
                "{person_id}"
            );
            assert_eq!(
                can_write(person_id, &PermissionItem::Entities, 1),
                chemistry_entity,
                "{person_id}"
            );
        }
    }

    #[test]
    fn test_profiles_can_read_restricted_products() {
        let db = crate::test_utils::init_test_with_fixture();

        for (person_id, expected) in [
            (ADMIN, true),
            (CHEMISTRY_MANAGER, true),
            (CHEMISTRY_USER, false),
            (BIOLOGY_MANAGER, true),
            (BIOLOGY_USER, true),
            (NEW_USER, false),
            (ALL_READER, false),
            (PRODUCT_EDITOR, true),
        ] {
            assert_eq!(
                person_can_read_restricted_products(&db, person_id).unwrap(),
                expected,
                "{person_id}"
            );
        }
    }
}
