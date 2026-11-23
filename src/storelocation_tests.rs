#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::storelocation::*;
    use crate::test_utils::profiles::*;

    // See the fixture description in resources/test_fixture.sql:
    //   Chemistry  1 Room 101 > 2 Cabinet [CA] > 3 Shelf
    //                         > 4 Fridge
    //   Biology    5 Room 201 > 6 Cabinet [CB]
    //                         > 7 Freezer (empty)

    fn count(db: &Connection, sql: &str, params: impl rusqlite::Params) -> i64 {
        db.query_row(sql, params, |row| row.get(0)).unwrap()
    }

    // The names of the store locations the admin gets with the filter, in
    // the order of the list: by name.
    fn names(db: &Connection, filter: &RequestFilter) -> Vec<String> {
        let (store_locations, count) = get_store_locations(db, filter, 1).unwrap();
        assert_eq!(count, store_locations.len());
        store_locations
            .into_iter()
            .map(|store_location| store_location.store_location_name)
            .collect()
    }

    // The color, parent and full path of a stored store location.
    fn saved(
        db: &Connection,
        store_location_id: u64,
    ) -> (Option<String>, Option<u64>, Option<String>) {
        db.query_row(
            "SELECT store_location_color, store_location, store_location_full_path FROM store_location WHERE store_location_id = ?1",
            [store_location_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
    }

    // A store location of Chemistry (1), named `name`, under `parent_id`;
    // `store_location_id` is `None` for a new one.
    fn store_location(
        store_location_id: Option<u64>,
        name: &str,
        parent_id: Option<u64>,
    ) -> StoreLocationStruct {
        StoreLocationStruct {
            store_location_id,
            store_location_name: name.to_string(),
            entity: Some(EntityStruct {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: parent_id.map(|parent_id| {
                Box::new(StoreLocationStruct {
                    store_location_id: Some(parent_id),
                    ..Default::default()
                })
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_get_store_locations_all() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(
            names(&db, &RequestFilter::default()),
            [
                "Cabinet [CA]",
                "Cabinet [CB]",
                "Freezer",
                "Fridge",
                "Room 101",
                "Room 201",
                "Shelf"
            ]
        );
    }

    #[test]
    fn test_get_store_locations_permissions() {
        let db = crate::test_utils::init_test_with_fixture();
        let chemistry = vec!["Cabinet [CA]", "Fridge", "Room 101", "Shelf"];
        let biology = vec!["Cabinet [CB]", "Freezer", "Room 201"];
        let all = vec![
            "Cabinet [CA]",
            "Cabinet [CB]",
            "Freezer",
            "Fridge",
            "Room 101",
            "Room 201",
            "Shelf",
        ];

        // Reading the store locations needs a storages permission, on their
        // entity or on all the entities.
        for (person_id, expected) in [
            (ADMIN, all.clone()),
            (CHEMISTRY_MANAGER, chemistry.clone()),
            (CHEMISTRY_USER, chemistry),
            (BIOLOGY_MANAGER, biology.clone()),
            (BIOLOGY_USER, biology),
            (NEW_USER, vec![]),
            (ALL_READER, all),
            (PRODUCT_EDITOR, vec![]),
        ] {
            let (store_locations, count) =
                get_store_locations(&db, &RequestFilter::default(), person_id).unwrap();
            let found: Vec<&str> = store_locations
                .iter()
                .map(|store_location| store_location.store_location_name.as_str())
                .collect();
            assert_eq!(count, expected.len(), "{person_id}");
            assert_eq!(found, expected, "{person_id}");
        }

        // A filter does not give more than what the person can read.
        let filter = RequestFilter {
            entity: Some(1),
            ..Default::default()
        };
        let (store_locations, _) = get_store_locations(&db, &filter, BIOLOGY_USER).unwrap();
        assert_eq!(store_locations.len(), 0);
    }

    #[test]
    fn test_get_store_locations_by_entity() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            entity: Some(1),
            ..Default::default()
        };
        assert_eq!(
            names(&db, &filter),
            ["Cabinet [CA]", "Fridge", "Room 101", "Shelf"]
        );
    }

    #[test]
    fn test_get_store_locations_by_parent() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            store_location: Some(1),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter), ["Cabinet [CA]", "Fridge"]);
    }

    #[test]
    fn test_get_store_locations_by_name() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            search: Some("Cabinet".to_string()),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter), ["Cabinet [CA]", "Cabinet [CB]"]);

        // The search also matches the entity name.
        let filter = RequestFilter {
            search: Some("Biology".to_string()),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter), ["Cabinet [CB]", "Freezer", "Room 201"]);
    }

    #[test]
    fn test_get_store_locations_combined_filters() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            entity: Some(2),
            search: Some("Cabinet".to_string()),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter), ["Cabinet [CB]"]);

        let filter = RequestFilter {
            entity: Some(1),
            store_location_can_store: true,
            ..Default::default()
        };
        assert_eq!(names(&db, &filter), ["Cabinet [CA]", "Fridge", "Shelf"]);
    }

    #[test]
    fn test_get_store_locations_counts() {
        let db = crate::test_utils::init_test_with_fixture();

        let (store_locations, _) = get_store_locations(&db, &RequestFilter::default(), 1).unwrap();
        let store_location = |name: &str| {
            store_locations
                .iter()
                .find(|store_location| store_location.store_location_name == name)
                .unwrap()
        };

        // Shelf: storages 1 and 2, not the history row 4.
        assert_eq!(store_location("Shelf").store_location_nb_storages, Some(2));
        assert_eq!(
            store_location("Room 101").store_location_nb_children,
            Some(2)
        );
        assert_eq!(
            store_location("Shelf").store_location_full_path.as_deref(),
            Some("Room 101/Cabinet [CA]/Shelf")
        );
    }

    #[test]
    fn test_get_store_locations_empty_result() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            entity: Some(999),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter).len(), 0);

        let filter = RequestFilter {
            search: Some("NonExistentLocation".to_string()),
            ..Default::default()
        };
        assert_eq!(names(&db, &filter).len(), 0);
    }

    #[test]
    fn test_create_store_location() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let mut bench = store_location(None, "Bench", Some(1));
        bench.store_location_can_store = true;
        bench.store_location_color = Some("#00ff00".to_string());
        let store_location_id = create_update_store_location(&mut db, bench).unwrap();

        assert_eq!(store_location_id, 8);
        assert_eq!(
            saved(&db, 8),
            (
                Some("#00ff00".to_string()),
                Some(1),
                Some("Room 101/Bench".to_string())
            )
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM store_location WHERE store_location_id = 8 AND store_location_can_store = 1",
                []
            ),
            1
        );
    }

    #[test]
    fn test_store_location_can_store_flag() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let store_location_id =
            create_update_store_location(&mut db, store_location(None, "Corridor", None)).unwrap();

        assert_eq!(
            count(
                &db,
                "SELECT store_location_can_store FROM store_location WHERE store_location_id = ?1",
                [store_location_id]
            ),
            0
        );
    }

    #[test]
    fn test_store_location_color_field() {
        let mut db = crate::test_utils::init_test_with_fixture();

        for color in ["#ff0000", "#00ff00", "#0000ff"] {
            let mut location = store_location(None, &format!("Box {color}"), Some(2));
            location.store_location_color = Some(color.to_string());
            let store_location_id = create_update_store_location(&mut db, location).unwrap();

            assert_eq!(saved(&db, store_location_id).0.as_deref(), Some(color));
        }
    }

    #[test]
    fn test_update_store_location() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let mut room = store_location(Some(1), "Room 102", None);
        room.store_location_color = Some("#ff0000".to_string());
        assert_eq!(create_update_store_location(&mut db, room).unwrap(), 1);

        assert_eq!(
            saved(&db, 1),
            (
                Some("#ff0000".to_string()),
                None,
                Some("Room 102".to_string())
            )
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM store_location WHERE store_location_id = 1 AND store_location_name = 'Room 102'",
                []
            ),
            1
        );
    }

    #[test]
    fn test_create_store_location_with_invalid_parent() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let result = create_update_store_location(&mut db, store_location(None, "Lost", Some(999)));

        assert!(result.is_err());
    }

    #[test]
    fn test_update_nonexistent_store_location() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Nothing to update: no error, and nothing created.
        let result =
            create_update_store_location(&mut db, store_location(Some(999), "Nowhere", None));

        assert!(result.is_ok());
        assert_eq!(count(&db, "SELECT COUNT(*) FROM store_location", []), 7);
    }

    #[test]
    fn test_delete_store_location() {
        let db = crate::test_utils::init_test_with_fixture();

        // Freezer (7) has no child and no storage.
        delete_store_location(&db, 7).unwrap();

        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM store_location WHERE store_location_id = 7",
                []
            ),
            0
        );
    }

    #[test]
    fn test_delete_store_location_with_history_rows() {
        let db = crate::test_utils::init_test_with_fixture();
        // A history row of storage 2 in Freezer (7): storage 2 was moved.
        db.execute(
            "INSERT INTO storage (person, product, store_location, storage) VALUES (3, 2, 7, 2)",
            [],
        )
        .unwrap();

        // History rows do not block the deletion: they are deleted with it.
        assert!(!store_location_has_storages(&db, 7).unwrap());
        delete_store_location(&db, 7).unwrap();

        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM storage WHERE store_location = 7",
                []
            ),
            0
        );
    }

    #[test]
    fn test_delete_nonexistent_store_location() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(delete_store_location(&db, 999).is_ok());
    }

    #[test]
    fn test_delete_store_location_with_children() {
        let db = crate::test_utils::init_test_with_fixture();

        // Cabinet [CA] (2) has a child, Shelf (3).
        assert!(delete_store_location(&db, 2).is_err());
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM store_location WHERE store_location_id = 2",
                []
            ),
            1
        );
    }

    #[test]
    fn test_delete_store_location_with_storages() {
        let db = crate::test_utils::init_test_with_fixture();

        // Cabinet [CB] (6) has storages 6 and 7.
        assert!(delete_store_location(&db, 6).is_err());
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM store_location WHERE store_location_id = 6",
                []
            ),
            1
        );
    }

    #[test]
    fn test_store_location_full_path_calculation() {
        let db = crate::test_utils::init_test_with_fixture();

        let mut location = store_location(None, "Drawer", Some(3));
        populate_store_location_full_path(&db, &mut location).unwrap();

        assert_eq!(
            location.store_location_full_path.as_deref(),
            Some("Room 101/Cabinet [CA]/Shelf/Drawer")
        );
    }

    #[test]
    fn test_store_location_has_storages_and_children() {
        let db = crate::test_utils::init_test_with_fixture();

        // Shelf (3) has storages, Room 101 (1) and Freezer (7) none.
        assert!(store_location_has_storages(&db, 3).unwrap());
        assert!(!store_location_has_storages(&db, 1).unwrap());
        assert!(!store_location_has_storages(&db, 7).unwrap());
        assert!(store_location_has_children(&db, 2).unwrap());
        assert!(!store_location_has_children(&db, 3).unwrap());
    }

    #[test]
    fn test_store_location_is_ancestor_of() {
        let db = crate::test_utils::init_test_with_fixture();

        // 1 > 2 > 3: 1 and 2 are ancestors of 3, and 3 is itself.
        assert!(store_location_is_ancestor_of(&db, 1, 3).unwrap());
        assert!(store_location_is_ancestor_of(&db, 2, 3).unwrap());
        assert!(store_location_is_ancestor_of(&db, 3, 3).unwrap());
        // 3 is a child of 2, not an ancestor; Fridge (4) is in another branch.
        assert!(!store_location_is_ancestor_of(&db, 3, 2).unwrap());
        assert!(!store_location_is_ancestor_of(&db, 4, 3).unwrap());
    }

    #[test]
    fn test_store_location_entity_id() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(store_location_entity_id(&db, 3).unwrap(), Some(1));
        assert_eq!(store_location_entity_id(&db, 6).unwrap(), Some(2));
        assert_eq!(store_location_entity_id(&db, 999).unwrap(), None);
    }

    #[test]
    fn test_update_store_location_clears_the_color() {
        let mut db = crate::test_utils::init_test_with_fixture();
        assert_eq!(saved(&db, 4).0.as_deref(), Some("#0000ff"));

        // Fridge (4) saved without color.
        create_update_store_location(&mut db, store_location(Some(4), "Fridge", Some(1))).unwrap();

        assert_eq!(saved(&db, 4).0, None);
    }

    #[test]
    fn test_update_store_location_moves_to_the_root() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Fridge (4) saved without parent.
        create_update_store_location(&mut db, store_location(Some(4), "Fridge", None)).unwrap();

        assert_eq!(saved(&db, 4), (None, None, Some("Fridge".to_string())));
    }

    #[test]
    fn test_update_store_location_updates_descendant_full_paths() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let full_path = |db: &Connection, store_location_id| saved(db, store_location_id).2;

        // Room 101 (1) renamed.
        create_update_store_location(&mut db, store_location(Some(1), "Room 102", None)).unwrap();
        assert_eq!(full_path(&db, 2).as_deref(), Some("Room 102/Cabinet [CA]"));
        assert_eq!(
            full_path(&db, 3).as_deref(),
            Some("Room 102/Cabinet [CA]/Shelf")
        );
        assert_eq!(full_path(&db, 4).as_deref(), Some("Room 102/Fridge"));

        // Cabinet [CA] (2) moved under Fridge (4), with its child Shelf (3).
        create_update_store_location(&mut db, store_location(Some(2), "Cabinet [CA]", Some(4)))
            .unwrap();
        assert_eq!(
            full_path(&db, 2).as_deref(),
            Some("Room 102/Fridge/Cabinet [CA]")
        );
        assert_eq!(
            full_path(&db, 3).as_deref(),
            Some("Room 102/Fridge/Cabinet [CA]/Shelf")
        );
        // Biology is not touched.
        assert_eq!(full_path(&db, 6).as_deref(), Some("Room 201/Cabinet [CB]"));
    }
}
