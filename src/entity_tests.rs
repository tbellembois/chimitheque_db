#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::entity::*;
    use rusqlite::Connection;
    use std::collections::HashMap;

    fn init_test_entity() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

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
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (7, 'person7@example.com')",
            [],
        )
        .unwrap();

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
    fn test_get_entities() {
        let db_connection = init_test_entity();

        let expected_nb_results_for_person =
            HashMap::from([(1, 10), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)]);

        for (person_id, expected_nb_results) in expected_nb_results_for_person {
            let (_, nb_resuts) = get_entities(
                &db_connection,
                RequestFilter {
                    ..Default::default()
                },
                person_id,
            )
            .unwrap();
            assert_eq!(nb_resuts, expected_nb_results);
        }
    }

    #[test]
    fn test_get_entities_sorting() {
        let db_connection = init_test_entity();

        // Test ascending order
        let ascending_filter = RequestFilter {
            order: "asc".to_string(),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, ascending_filter, 1).unwrap();
        let first_entity_name = &entities[0].entity_name;
        let second_entity_name = &entities[1].entity_name;
        assert!(
            first_entity_name < second_entity_name,
            "Entities should be in ascending order. Entities: {entities:?}"
        );

        // Test descending order
        let descending_filter = RequestFilter {
            order: "desc".to_string(),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, descending_filter, 1).unwrap();
        let first_entity_name = &entities[0].entity_name;
        let second_entity_name = &entities[1].entity_name;
        assert!(
            first_entity_name > second_entity_name,
            "Entities should be in descending order. Entities: {entities:?}"
        );
    }

    #[test]
    fn test_get_entities_filtering() {
        let db_connection = init_test_entity();

        // Test filtering entities by name
        let by_name_filter = RequestFilter {
            entity_name: Some("Chemistry Department".to_string()),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, by_name_filter, 1).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(&entities[0].entity_name, "Chemistry Department");

        // Test filtering entities by id
        let by_id_filter = RequestFilter {
            id: Some(2),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, by_id_filter, 1).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_id.unwrap(), 2);

        // Test filtering entities by search query
        let by_search_filter = RequestFilter {
            search: Some("Physics".to_string()),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, by_search_filter, 1).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(&entities[0].entity_name, "Physics Department");
    }

    #[test]
    fn test_get_entities_managers() {
        let db_connection = init_test_entity();

        // Person 1 has permission to see all entities
        let filter = RequestFilter {
            ..Default::default()
        };
        let (entities, _) = get_entities(&db_connection, filter, 1).unwrap();

        // We check specific entities that we know have managers in init_test_entity
        // Entity 1 -> Person 2 (person2@example.com)
        // Entity 2 -> Person 3 (person3@example.com)
        // Entity 3 -> Person 4 (person4@example.com)

        let entity1 = entities
            .iter()
            .find(|e| e.entity_id == Some(1))
            .expect("Entity 1 should exist");
        let managers1 = entity1
            .managers
            .as_ref()
            .expect("Entity 1 should have managers");
        assert_eq!(managers1.len(), 1);
        assert_eq!(managers1[0].person_id, Some(2));
        assert_eq!(managers1[0].person_email, "person2@example.com");

        let entity2 = entities
            .iter()
            .find(|e| e.entity_id == Some(2))
            .expect("Entity 2 should exist");
        let managers2 = entity2
            .managers
            .as_ref()
            .expect("Entity 2 should have managers");
        assert_eq!(managers2.len(), 1);
        assert_eq!(managers2[0].person_id, Some(3));
        assert_eq!(managers2[0].person_email, "person3@example.com");

        // Check an entity that has NO manager assigned in init_test_entity (e.g., Entity 10)
        let entity10 = entities
            .iter()
            .find(|e| e.entity_id == Some(10))
            .expect("Entity 10 should exist");
        assert!(
            entity10.managers.is_none(),
            "Entity 10 should not have any managers"
        );
    }

    #[test]
    fn test_create_entity() {
        let mut db_connection = init_test_entity();

        let new_entity = chimitheque_types::entity::Entity {
            entity_id: None,
            entity_name: "New Lab".to_string(),
            entity_description: Some("Testing creation".to_string()),
            managers: Some(vec![chimitheque_types::person::Person {
                person_id: Some(1),
                person_email: "person1@example.com".to_string(),
                ..Default::default()
            }]),
            entity_nb_store_locations: Some(0),
            entity_nb_people: Some(0),
        };

        let id = create_update_entity(&mut db_connection, new_entity).unwrap();
        assert!(id > 10); // Should be greater than the 10 we inserted in init

        // Verify it exists in DB
        let mut stmt = db_connection
            .prepare("SELECT entity_name FROM entity WHERE entity_id = ?")
            .unwrap();
        let name: String = stmt.query_row([id], |row| row.get(0)).unwrap();
        assert_eq!(name, "New Lab");

        // Verify manager was linked
        let mut stmt = db_connection
            .prepare(
                "SELECT entitypeople_person_id FROM entitypeople WHERE entitypeople_entity_id = ?",
            )
            .unwrap();
        let manager_id: u64 = stmt.query_row([id], |row| row.get(0)).unwrap();
        assert_eq!(manager_id, 1);
    }

    #[test]
    fn test_create_update_entity_managers() {
        let mut db_connection = init_test_entity();

        // We need a transaction because the function requires &Transaction
        let tx = db_connection.transaction().unwrap();

        let entity = chimitheque_types::entity::Entity {
            entity_id: Some(1), // Use existing entity 1
            entity_name: "Test".to_string(),
            entity_description: Some("Test".to_string()),
            managers: Some(vec![
                chimitheque_types::person::Person {
                    person_id: Some(6),
                    person_email: "person6@example.com".to_string(),
                    ..Default::default()
                },
                chimitheque_types::person::Person {
                    person_id: Some(7),
                    person_email: "person7@example.com".to_string(),
                    ..Default::default()
                },
            ]),
            entity_nb_store_locations: Some(0),
            entity_nb_people: Some(0),
        };

        // Call the private function directly
        create_update_entity_managers(&tx, &entity).unwrap();
        tx.commit().unwrap();

        // Verify that Entity 1 now has exactly 2 managers: 6 and 7
        let mut stmt = db_connection.prepare("SELECT entitypeople_person_id FROM entitypeople WHERE entitypeople_entity_id = 1 ORDER BY entitypeople_person_id ASC").unwrap();
        let ids: Vec<u64> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(ids, vec![6, 7]);
    }

    #[test]
    fn test_update_entity() {
        let mut db_connection = init_test_entity();

        let update_entity = chimitheque_types::entity::Entity {
            entity_id: Some(1),
            entity_name: "Updated Chemistry Dept".to_string(),
            entity_description: Some("Updated description".to_string()),
            managers: Some(vec![chimitheque_types::person::Person {
                person_id: Some(5), // Change manager from 2 to 5
                person_email: "person5@example.com".to_string(),
                ..Default::default()
            }]),
            entity_nb_store_locations: Some(0),
            entity_nb_people: Some(0),
        };

        let id = create_update_entity(&mut db_connection, update_entity).unwrap();
        assert_eq!(id, 1);

        // Verify name updated
        let mut stmt = db_connection
            .prepare("SELECT entity_name FROM entity WHERE entity_id = 1")
            .unwrap();
        let name: String = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(name, "Updated Chemistry Dept");

        // Verify manager updated (old one gone, new one present)
        let mut stmt = db_connection
            .prepare(
                "SELECT entitypeople_person_id FROM entitypeople WHERE entitypeople_entity_id = 1",
            )
            .unwrap();
        let manager_id: u64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(manager_id, 5);
    }

    #[test]
    fn test_delete_entity() {
        let mut db_connection = init_test_entity();
        let target_id = 10;

        delete_entity(&mut db_connection, target_id).unwrap();

        // Verify it is gone
        let mut stmt = db_connection
            .prepare("SELECT count(*) FROM entity WHERE entity_id = ?")
            .unwrap();
        let count: i32 = stmt.query_row([target_id], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
