#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::permission::{person_can_read_restricted_products, person_can_write_in_entity};
    use chimitheque_types::permission::PermissionItem;
    use rusqlite::Connection;

    // Person 1 has the given permissions, as (name, item, entity) tuples.
    fn init_test_permissions(permissions: &[(&str, &str, Option<u64>)]) -> Connection {
        let db = crate::test_utils::init_test();
        db.execute_batch(
            "INSERT INTO person (person_id, person_email) VALUES (1, 'test@example.com');
             INSERT INTO entity (entity_id, entity_name) VALUES (1, 'entity 1'), (2, 'entity 2');",
        )
        .unwrap();
        for (name, item, entity) in permissions {
            db.execute(
                "INSERT INTO permission (person, permission_name, permission_item, permission_entity) VALUES (1, ?1, ?2, ?3)",
                (name, item, entity),
            )
            .unwrap();
        }
        db
    }

    fn can_write_storages(db: &Connection, entity_id: u64) -> bool {
        person_can_write_in_entity(db, 1, &PermissionItem::Storages, entity_id).unwrap()
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
        assert!(person_can_write_in_entity(&db, 1, &PermissionItem::Entities, 1).unwrap());
    }

    #[test]
    fn test_person_can_read_restricted_products() {
        let can_read = |permissions: &[(&str, &str, Option<u64>)]| {
            let db = init_test_permissions(permissions);
            person_can_read_restricted_products(&db, 1).unwrap()
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
}
