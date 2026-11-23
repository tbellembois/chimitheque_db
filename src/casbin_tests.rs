#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::casbin::*;
    use crate::test_utils::profiles::*;

    // See the fixture description in resources/test_fixture.sql.

    #[test]
    fn test_match_person_is_in_entity() {
        let db = crate::test_utils::init_test_with_fixture();

        // chemistry.user is a member of Chemistry (1), not of Biology (2).
        assert!(match_person_is_in_entity(&db, 3, 1).unwrap());
        assert!(!match_person_is_in_entity(&db, 3, 2).unwrap());
        // The admin, and the people with no entity (to be given one), are in
        // all the entities.
        for person_id in [ADMIN, NEW_USER, ALL_READER, PRODUCT_EDITOR] {
            assert!(
                match_person_is_in_entity(&db, person_id, 2).unwrap(),
                "{person_id}"
            );
        }
        // An unknown person is not a person with no entity.
        assert!(match_person_is_in_entity(&db, 999, 2).is_err());
    }

    #[test]
    fn test_match_person_is_admin_or_manager() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(match_person_is_admin(&db, 1).unwrap());
        assert!(!match_person_is_admin(&db, 2).unwrap());
        assert!(!match_person_is_admin(&db, 999).unwrap());

        assert!(match_person_is_manager(&db, 2).unwrap());
        assert!(match_person_is_manager(&db, 4).unwrap());
        assert!(!match_person_is_manager(&db, 3).unwrap());
        assert!(!match_person_is_manager(&db, 1).unwrap());
    }

    #[test]
    fn test_match_product_has_storages() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(match_product_has_storages(&db, 2).unwrap());
        assert!(match_product_has_storages(&db, 4).unwrap());
        assert!(!match_product_has_storages(&db, 3).unwrap());
        assert!(!match_product_has_storages(&db, 6).unwrap());
    }

    #[test]
    fn test_match_store_location_children_and_storages() {
        let db = crate::test_utils::init_test_with_fixture();

        // Room 101 (1) and Cabinet [CA] (2) have children, Shelf (3) and
        // Freezer (7) do not.
        assert!(match_store_location_has_children(&db, 1).unwrap());
        assert!(match_store_location_has_children(&db, 2).unwrap());
        assert!(!match_store_location_has_children(&db, 3).unwrap());
        assert!(!match_store_location_has_children(&db, 7).unwrap());

        // Shelf (3) and Cabinet [CB] (6) have storages, Room 101 (1) and
        // Freezer (7) do not.
        assert!(match_store_location_has_storages(&db, 3).unwrap());
        assert!(match_store_location_has_storages(&db, 6).unwrap());
        assert!(!match_store_location_has_storages(&db, 1).unwrap());
        assert!(!match_store_location_has_storages(&db, 7).unwrap());
    }

    #[test]
    fn test_match_entity_members_and_store_locations() {
        let db = crate::test_utils::init_test_with_fixture();

        // Physics (3) has no member and no store location.
        assert!(match_entity_has_members(&db, 1).unwrap());
        assert!(!match_entity_has_members(&db, 3).unwrap());
        assert!(match_entity_has_store_locations(&db, 1).unwrap());
        assert!(!match_entity_has_store_locations(&db, 3).unwrap());
    }

    #[test]
    fn test_match_is_in_entity() {
        let db = crate::test_utils::init_test_with_fixture();

        // Shelf (3) is in Chemistry (1).
        assert!(match_store_location_is_in_entity(&db, 3, 1).unwrap());
        assert!(!match_store_location_is_in_entity(&db, 3, 2).unwrap());
        assert!(!match_store_location_is_in_entity(&db, 999, 1).unwrap());

        // Storage 1 is in Chemistry (1), storage 6 in Biology (2).
        assert!(match_storage_is_in_entity(&db, 1, 1).unwrap());
        assert!(match_storage_is_in_entity(&db, 6, 2).unwrap());
        assert!(!match_storage_is_in_entity(&db, 1, 2).unwrap());
        assert!(!match_storage_is_in_entity(&db, 999, 1).unwrap());
    }

    #[test]
    fn test_to_string_adapter() {
        let db = crate::test_utils::init_test_with_fixture();

        let policy = to_string_adapter(&db).unwrap();

        // An entity is written empty for all the entities.
        assert!(policy.contains("p, 1, all, all, \n"));
        assert!(policy.contains("p, 2, all, all, 1\n"));
        assert!(policy.contains("p, 3, w, storages, 1\n"));
        assert!(policy.contains("p, 6, r, products, \n"));
        assert!(policy.contains("p, 7, r, storages, \n"));
        assert!(policy.contains("p, 8, w, products, \n"));
    }
}
