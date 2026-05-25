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
}
