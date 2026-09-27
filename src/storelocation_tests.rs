#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::storelocation::*;

    fn init_test_storelocation() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Delete existing records if any
        db.execute("DELETE FROM borrowing", []).unwrap();

        // Insert example data into the person table
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (1, 'person1@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (2, 'person2@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (3, 'person3@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (4, 'person4@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (5, 'person5@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (6, 'person6@example.com')",
            [],
        )
        .unwrap();

        // Insert example data into the cas_number table
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (1, '7732-18-5')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (2, '7782-39-0')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (3, '7758-99-8')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (4, '7757-82-6')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (5, '7783-41-7')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (6, '7785-90-8')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (7, '7789-20-0')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (8, '7782-44-7')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (9, '7783-40-6')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO cas_number (cas_number_id, cas_number_label) VALUES (10, '7784-42-1')",
            [],
        )
        .unwrap();

        // Insert example data into the empirical_formula table
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (1, 'H2O')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (2, 'NaCl')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (3, 'C6H12O6')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (4, 'CH4')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (5, 'CO2')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (6, 'NH3')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (7, 'C2H5OH')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (8, 'H2SO4')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (9, 'CaCO3')",
        [],
    )
    .unwrap();
        db.execute("INSERT INTO empirical_formula (empirical_formula_id, empirical_formula_label) VALUES (10, 'KMnO4')",
        [],
    )
    .unwrap();

        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (1, 'Water')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (2, 'Sodium Chloride')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (3, 'Glucose')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (4, 'Methane')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (5, 'Carbon Dioxide')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (6, 'Ammonia')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (7, 'Ethanol')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (8, 'Sulfuric Acid')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (9, 'Calcium Carbonate')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO name (name_id, name_label) VALUES (10, 'Potassium Permanganate')",
            [],
        )
        .unwrap();

        // Insert example data into the product table
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (1, 1, 'chem', 1, 1)"
            , []).unwrap();
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (2, 2, 'chem', 2, 2)"
            , []).unwrap();
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (3, 3, 'chem', 3, 3)"
            , []).unwrap();
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (4, 4, 'chem', 4, 4)"
            , []).unwrap();
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (5, 5, 'chem', 5, 5)"
            , []).unwrap();
        db.execute(
            "INSERT INTO product (product_id, name, product_type, cas_number, empirical_formula) VALUES (6, 6, 'chem', 6, 6)"
            , []).unwrap();

        // Insert example data into the entity table
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (1, 'Chemistry Department', 'Department of Chemistry')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (2, 'Physics Department', 'Department of Physics')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (3, 'Biology Department', 'Department of Biology')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (4, 'Main Laboratory', 'Main research laboratory')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (5, 'Analytical Lab', 'Lab for analytical chemistry')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (6, 'Organic Chemistry Lab', 'Lab for organic chemistry research')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (7, 'Inorganic Chemistry Lab', 'Lab for inorganic chemistry research')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (8, 'Physical Chemistry Lab', 'Lab for physical chemistry research')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (9, 'Storage Facility', 'Main storage facility for chemicals')"
            , []).unwrap();
        db.execute(
            "INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (10, 'Safety Office', 'Office responsible for safety and compliance')"
            , []).unwrap();

        // Insert example data into the store_location table
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (1, 'Main Storage', 1, NULL)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (2, 'Chemical Storage', 1, 1)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (3, 'Flammable Storage', 1, 2)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (4, 'Corrosive Storage', 1, 2)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (5, 'Toxic Storage', 1, 2)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (6, 'Cold Storage', 1, 1)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (7, 'Refrigerated Storage', 1, 6)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (8, 'Freezer Storage', 1, 6)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (9, 'Lab 1 Storage', 2, 1)"
            , []).unwrap();
        db.execute(
            "INSERT INTO store_location (store_location_id, store_location_name, entity, store_location) VALUES (10, 'Lab 2 Storage', 2, 1)"
            , []).unwrap();

        // Insert example data into the storage table
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (1, 1, 1, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (2, 1, 2, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (3, 2, 1, 3)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (4, 2, 3, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (5, 3, 2, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (6, 3, 4, 3)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (7, 4, 1, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (8, 4, 3, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (9, 5, 2, 3)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO storage (storage_id, person, product, store_location) VALUES (10, 5, 4, 1)"
            , []).unwrap();

        db.execute("INSERT INTO entitypeople VALUES (1, 2),(2, 3),(3, 4)", [])
            .unwrap();
        db.execute(
            "INSERT INTO personentities VALUES
        (2,1),
        (3,2),
        (4,3),
        (5,1),
        (6,2),
        (7,3)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO permission VALUES (1,'all','all',NULL),
            (2,'all','all',1),
            (3,'all','all',2),
            (4,'all','all',3),
            (5,'n','rproducts',NULL),
            (5,'n','storages',1),
            (5,'r','products',NULL),
            (5,'r','entities',1),
            (6,'w','products',NULL),
            (6,'r','rproducts',NULL),
            (6,'r','storages',2),
            (6,'r','products',NULL),
            (6,'r','entities',2),
            (7,'w','products',NULL),
            (7,'r','rproducts',NULL),
            (7,'w','storages',3),
            (7,'r','products',NULL),
            (7,'r','entities',3)",
            [],
        )
        .unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_get_store_locations_all() {
        let db = init_test_storelocation();

        // Get all store locations
        let (locations, count) = get_store_locations(&db, &RequestFilter::default(), 1).unwrap();

        // Verify we got all 10 locations
        assert_eq!(count, 10);
        assert_eq!(locations.len(), 10);
    }

    #[test]
    fn test_get_store_locations_by_entity() {
        let db = init_test_storelocation();

        // Get locations for Chemistry Department (entity_id = 1)
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                entity: Some(1),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        // Verify we got the correct number of locations for this entity
        // Main Storage (1) and its sub-locations (2-8)
        assert_eq!(count, 8);
        assert_eq!(locations.len(), 8);

        // Check that all locations belong to the correct entity
        for location in locations {
            assert_eq!(location.entity.unwrap().entity_id, Some(1));
        }
    }

    #[test]
    fn test_get_store_locations_by_parent() {
        let db = init_test_storelocation();

        // Get locations where parent is Chemical Storage (store_location_id = 2)
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                store_location: Some(2),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        // Verify we got the sub-locations of Chemical Storage
        // Flammable Storage (3), Corrosive Storage (4), Toxic Storage (5)
        assert_eq!(count, 3);
        assert_eq!(locations.len(), 3);

        // Check that all locations have the correct parent
        for location in locations {
            assert_eq!(location.store_location.unwrap().store_location_id, Some(2));
        }
    }

    #[test]
    fn test_get_store_locations_by_name() {
        let db = init_test_storelocation();

        // Search for locations containing "Storage" in their name
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                search: Some("Storage".to_string()),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        // Verify we got all locations (all contain "Storage" in their name)
        assert_eq!(count, 10);
        assert_eq!(locations.len(), 10);

        // Search for locations containing "Cold" in their name
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                search: Some("Cold".to_string()),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        // Verify we got only Cold Storage and its sub-locations
        assert_eq!(count, 1);
        assert_eq!(locations.len(), 1);
    }

    #[test]
    fn test_get_store_locations_combined_filters() {
        let db = init_test_storelocation();

        // Get locations for Physics Department (entity_id = 2) that contain "Lab" in their name
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                entity: Some(2),
                search: Some("Lab".to_string()),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        // Verify we got the correct locations
        assert_eq!(count, 2);
        assert_eq!(locations.len(), 2);

        // Check that all locations belong to the correct entity and contain "Lab" in their name
        for location in locations {
            assert_eq!(location.entity.unwrap().entity_id.unwrap(), 2);
            assert!(location.store_location_name.contains("Lab"));
        }
    }

    #[test]
    fn test_create_store_location() {
        let mut db = init_test_storelocation();

        let new_location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: None,
            store_location_name: "New test location".to_string(),
            store_location_can_store: true,
            store_location_color: Some("blue".to_string()),
            store_location_full_path: None,
            entity: Some(chimitheque_types::entity::Entity {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: Some(Box::new(chimitheque_types::storelocation::StoreLocation {
                store_location_id: Some(1), // Parent is Main Storage
                ..Default::default()
            })),
            store_location_nb_storages: Some(0),
            store_location_nb_children: Some(0),
        };

        let id = create_update_store_location(&mut db, new_location).unwrap();
        assert!(id > 10);

        // Verify it exists and path was populated
        let mut stmt = db.prepare("SELECT store_location_name, store_location_full_path FROM store_location WHERE store_location_id = ?").unwrap();
        let (name, path): (String, String) = stmt
            .query_row([id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();

        assert_eq!(name, "New test location");
        // Path should be "Main Storage/New test location" (since parent 1 is "Main Storage")
        assert!(path.contains("Main Storage"));
        assert!(path.contains("New test location"));
    }

    #[test]
    fn test_update_store_location() {
        let mut db = init_test_storelocation();

        let update_location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: Some(1),
            store_location_name: "Updated Main Storage".to_string(),
            store_location_can_store: false,
            store_location_color: Some("red".to_string()),
            store_location_full_path: None, // Will be re-calculated
            entity: Some(chimitheque_types::entity::Entity {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: None,
            store_location_nb_storages: Some(0),
            store_location_nb_children: Some(0),
        };

        let id = create_update_store_location(&mut db, update_location).unwrap();
        assert_eq!(id, 1);

        // Verify changes
        let mut stmt = db.prepare("SELECT store_location_name, store_location_can_store, store_location_color FROM store_location WHERE store_location_id = 1").unwrap();
        let (name, can_store, color): (String, bool, String) = stmt
            .query_row([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap();

        assert_eq!(name, "Updated Main Storage");
        assert!(!can_store);
        assert_eq!(color, "red");
    }

    #[test]
    fn test_delete_store_location() {
        let db = init_test_storelocation();

        // Use location 10 which has no children and storages
        let target_id = 10;

        // Verify it exists first
        let mut stmt = db
            .prepare("SELECT count(*) FROM store_location WHERE store_location_id = ?")
            .unwrap();
        assert_eq!(
            stmt.query_row([target_id], |row| row.get::<_, i32>(0))
                .unwrap(),
            1
        );

        delete_store_location(&db, target_id).unwrap();

        // Verify it's gone
        let mut stmt = db
            .prepare("SELECT count(*) FROM store_location WHERE store_location_id = ?")
            .unwrap();
        assert_eq!(
            stmt.query_row([target_id], |row| row.get::<_, i32>(0))
                .unwrap(),
            0
        );

        // Verify associated storages that were linked to this location are also gone
        // (Because delete_store_location explicitly deletes storages first)
        let mut stmt = db
            .prepare("SELECT count(*) FROM storage WHERE store_location = ?")
            .unwrap();
        assert_eq!(
            stmt.query_row([target_id], |row| row.get::<_, i32>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_get_store_locations_empty_result() {
        let db = init_test_storelocation();

        // Search for non-existent entity
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                entity: Some(999), // Non-existent entity
                ..Default::default()
            },
            1,
        )
        .unwrap();

        assert_eq!(count, 0);
        assert_eq!(locations.len(), 0);
    }

    #[test]
    fn test_get_store_locations_empty_search() {
        let db = init_test_storelocation();

        // Search for non-existent term
        let (locations, count) = get_store_locations(
            &db,
            &RequestFilter {
                search: Some("NonExistentLocation".to_string()),
                ..Default::default()
            },
            1,
        )
        .unwrap();

        assert_eq!(count, 0);
        assert_eq!(locations.len(), 0);
    }

    #[test]
    fn test_create_store_location_with_invalid_parent() {
        let mut db = init_test_storelocation();

        let new_location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: None,
            store_location_name: "Orphan location".to_string(),
            store_location_can_store: true,
            store_location_color: Some("green".to_string()),
            store_location_full_path: None,
            entity: Some(chimitheque_types::entity::Entity {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: Some(Box::new(chimitheque_types::storelocation::StoreLocation {
                store_location_id: Some(999), // Non-existent parent
                ..Default::default()
            })),
            store_location_nb_storages: Some(0),
            store_location_nb_children: Some(0),
        };

        // Create should prevent creation
        let result = create_update_store_location(&mut db, new_location);
        assert!(result.is_err()); // Or should return Err if validation is implemented
    }

    #[test]
    fn test_update_nonexistent_store_location() {
        let mut db = init_test_storelocation();

        let update_location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: Some(999), // Non-existent ID
            store_location_name: "Should not update".to_string(),
            store_location_can_store: false,
            store_location_color: Some("black".to_string()),
            store_location_full_path: None,
            entity: Some(chimitheque_types::entity::Entity {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: None,
            store_location_nb_storages: Some(0),
            store_location_nb_children: Some(0),
        };

        // Should handle gracefully - either return error or do nothing
        let result = create_update_store_location(&mut db, update_location);
        assert!(result.is_ok()); // Or should assert!(result.is_err()) if proper error handling is expected
    }

    #[test]
    fn test_delete_nonexistent_store_location() {
        let db = init_test_storelocation();

        // Should not panic or error when trying to delete non-existent location
        let result = delete_store_location(&db, 999);
        assert!(result.is_ok()); // Or assert!(result.is_err()) if strict error handling is expected
    }

    #[test]
    fn test_delete_store_location_with_children() {
        let db = init_test_storelocation();

        // Location 2 has children (3, 4, 5)
        let target_id = 2;

        // Verify it has children before deletion
        let mut stmt = db
            .prepare("SELECT count(*) FROM store_location WHERE store_location = ?")
            .unwrap();
        let child_count: i32 = stmt.query_row([target_id], |row| row.get(0)).unwrap();
        assert!(child_count > 0);

        // Delete should prevent deletion
        let result = delete_store_location(&db, target_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_store_location_with_storages() {
        let db = init_test_storelocation();

        // Location 1 has storages (check storage table)
        let target_id = 1;

        // Verify it has storages before deletion
        let mut stmt = db
            .prepare("SELECT count(*) FROM storage WHERE store_location = ?")
            .unwrap();
        let storage_count: i32 = stmt.query_row([target_id], |row| row.get(0)).unwrap();
        assert!(storage_count > 0);

        // Delete should prevent deletion
        let result = delete_store_location(&db, target_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_store_location_can_store_flag() {
        let mut db = init_test_storelocation();

        let new_location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: None,
            store_location_name: "Read-only storage".to_string(),
            store_location_can_store: false, // Cannot store products
            store_location_color: Some("gray".to_string()),
            store_location_full_path: None,
            entity: Some(chimitheque_types::entity::Entity {
                entity_id: Some(1),
                ..Default::default()
            }),
            store_location: Some(Box::new(chimitheque_types::storelocation::StoreLocation {
                store_location_id: Some(1), // Parent is Main Storage
                ..Default::default()
            })),
            store_location_nb_storages: Some(0),
            store_location_nb_children: Some(0),
        };

        let id = create_update_store_location(&mut db, new_location).unwrap();

        // Verify the flag was set
        let mut stmt = db
            .prepare(
                "SELECT store_location_can_store FROM store_location WHERE store_location_id = ?",
            )
            .unwrap();
        let can_store: bool = stmt.query_row([id], |row| row.get(0)).unwrap();

        assert!(!can_store);
    }

    #[test]
    fn test_store_location_color_field() {
        let mut db = init_test_storelocation();

        let colors = vec!["red", "blue", "green", "yellow", "purple"];

        for color in colors {
            let new_location = chimitheque_types::storelocation::StoreLocation {
                store_location_id: None,
                store_location_name: format!("Color test: {color}"),
                store_location_can_store: true,
                store_location_color: Some(color.to_string()),
                store_location_full_path: None,
                entity: Some(chimitheque_types::entity::Entity {
                    entity_id: Some(1),
                    ..Default::default()
                }),
                store_location: None,
                store_location_nb_storages: Some(0),
                store_location_nb_children: Some(0),
            };

            let id = create_update_store_location(&mut db, new_location).unwrap();

            // Verify the color was set
            let mut stmt = db
                .prepare(
                    "SELECT store_location_color FROM store_location WHERE store_location_id = ?",
                )
                .unwrap();
            let stored_color: Option<String> = stmt.query_row([id], |row| row.get(0)).unwrap();

            assert_eq!(stored_color, Some(color.to_string()));
        }
    }

    #[test]
    fn test_store_location_full_path_calculation() {
        let db = init_test_storelocation();

        // Create a location with a parent hierarchy that will be queried from the database
        let mut location = chimitheque_types::storelocation::StoreLocation {
            store_location_id: Some(9),
            store_location_name: "Lab 1 Storage".to_string(),
            store_location_can_store: true,
            store_location_color: None,
            store_location_full_path: None, // Will be populated by the function
            entity: None,
            store_location: Some(Box::new(chimitheque_types::storelocation::StoreLocation {
                store_location_id: Some(1), // Parent is Main Storage
                store_location_name: "Main Storage".to_string(),
                ..Default::default()
            })),
            store_location_nb_storages: None,
            store_location_nb_children: None,
        };

        // Call the function directly without going through create_update_store_location
        let result = populate_store_location_full_path(&db, &mut location);
        assert!(result.is_ok());

        assert_eq!(
            location.store_location_full_path,
            Some("Main Storage/Lab 1 Storage".to_string())
        );
    }
}
