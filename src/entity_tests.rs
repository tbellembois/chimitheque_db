#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::entity::*;
    use chimitheque_types::{person::Person, sortorder::SortOrder};
    use rusqlite::Connection;

    // See the fixture description in resources/test_fixture.sql: Chemistry
    // (1, managed by 2), Biology (2, managed by 4), Physics (3, empty).

    fn count(db: &Connection, sql: &str) -> i64 {
        db.query_row(sql, [], |row| row.get(0)).unwrap()
    }

    fn names(entities: &[EntityStruct]) -> Vec<&str> {
        entities
            .iter()
            .map(|entity| entity.entity_name.as_str())
            .collect()
    }

    fn manager_ids(db: &Connection, entity_id: u64) -> Vec<u64> {
        let mut stmt = db
            .prepare("SELECT entitypeople_person_id FROM entitypeople WHERE entitypeople_entity_id = ?1 ORDER BY 1")
            .unwrap();
        stmt.query_map([entity_id], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<u64>, _>>()
            .unwrap()
    }

    fn person(person_id: u64) -> Person {
        Person {
            person_id: Some(person_id),
            ..Default::default()
        }
    }

    // new.user (6) becomes the manager, and so a member, of Physics (3).
    fn manage_physics(db: &mut Connection) {
        let tx = db.transaction().unwrap();
        crate::person::set_person_manager(&tx, 6, 3).unwrap();
        tx.commit().unwrap();
    }

    #[test]
    fn test_get_entities() {
        let db = crate::test_utils::init_test_with_fixture();

        // The admin reads all the entities, the others their entity, new.user
        // none.
        for (person_id, expected) in [
            (1, vec!["Biology", "Chemistry", "Physics"]),
            (2, vec!["Chemistry"]),
            (3, vec!["Chemistry"]),
            (4, vec!["Biology"]),
            (5, vec!["Biology"]),
            (6, vec![]),
        ] {
            let (entities, count) = get_entities(&db, RequestFilter::default(), person_id).unwrap();
            assert_eq!(count, expected.len(), "{person_id}");
            assert_eq!(names(&entities), expected, "{person_id}");
        }
    }

    #[test]
    fn test_get_entities_sorting() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            order: SortOrder::Asc,
            ..Default::default()
        };
        let (entities, _) = get_entities(&db, filter, 1).unwrap();
        assert_eq!(names(&entities), ["Biology", "Chemistry", "Physics"]);

        let filter = RequestFilter {
            order: SortOrder::Desc,
            ..Default::default()
        };
        let (entities, _) = get_entities(&db, filter, 1).unwrap();
        assert_eq!(names(&entities), ["Physics", "Chemistry", "Biology"]);
    }

    #[test]
    fn test_get_entities_filtering() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            entity_name: Some("Chemistry".to_string()),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db, filter, 1).unwrap();
        assert_eq!(names(&entities), ["Chemistry"]);

        let filter = RequestFilter {
            id: Some(2),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db, filter, 1).unwrap();
        assert_eq!(names(&entities), ["Biology"]);

        let filter = RequestFilter {
            search: Some("Phys".to_string()),
            ..Default::default()
        };
        let (entities, _) = get_entities(&db, filter, 1).unwrap();
        assert_eq!(names(&entities), ["Physics"]);
    }

    #[test]
    fn test_get_entities_empty_result() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            entity_name: Some("NonExistentDepartment".to_string()),
            ..Default::default()
        };
        let (entities, count) = get_entities(&db, filter, 1).unwrap();

        assert_eq!(count, 0);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_get_entities_managers_and_counts() {
        let db = crate::test_utils::init_test_with_fixture();

        let (entities, _) = get_entities(&db, RequestFilter::default(), 1).unwrap();
        let entity = |entity_id| {
            entities
                .iter()
                .find(|entity| entity.entity_id == Some(entity_id))
                .unwrap()
        };

        let managers = entity(1).managers.as_ref().unwrap();
        assert_eq!(managers.len(), 1);
        assert_eq!(managers[0].person_id, Some(2));
        assert_eq!(managers[0].person_email, "chemistry.manager@example.com");
        assert_eq!(entity(2).managers.as_ref().unwrap()[0].person_id, Some(4));
        assert!(entity(3).managers.is_none());

        // Chemistry: 2 members, 4 store locations.
        assert_eq!(entity(1).entity_nb_people, Some(2));
        assert_eq!(entity(1).entity_nb_store_locations, Some(4));
        assert_eq!(entity(3).entity_nb_people, Some(0));
    }

    #[test]
    fn test_create_entity() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let new_entity = EntityStruct {
            entity_name: "Mathematics".to_string(),
            entity_description: Some("Mathematics laboratory".to_string()),
            managers: Some(vec![person(3)]),
            ..Default::default()
        };
        let entity_id = create_update_entity(&mut db, new_entity).unwrap();

        assert_eq!(entity_id, 4);
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM entity WHERE entity_id = 4 AND entity_name = 'Mathematics'"
            ),
            1
        );
        // chemistry.user manages it, with the manager permission.
        assert_eq!(manager_ids(&db, 4), [3]);
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM permission WHERE person = 3 AND permission_name = 'all' AND permission_item = 'all' AND permission_entity = 4"
            ),
            1
        );
    }

    #[test]
    fn test_create_entity_unique_name_constraint() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let duplicate = EntityStruct {
            entity_name: "Chemistry".to_string(),
            ..Default::default()
        };

        assert!(create_update_entity(&mut db, duplicate).is_err());
    }

    #[test]
    fn test_create_update_entity_managers() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Chemistry is managed by chemistry.user and biology.user instead of
        // chemistry.manager.
        let entity = EntityStruct {
            entity_id: Some(1),
            entity_name: "Chemistry".to_string(),
            managers: Some(vec![person(3), person(5)]),
            ..Default::default()
        };
        let tx = db.transaction().unwrap();
        create_update_entity_managers(&tx, &entity).unwrap();
        tx.commit().unwrap();

        assert_eq!(manager_ids(&db, 1), [3, 5]);
        // The manager permission of the former manager is removed.
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM permission WHERE person = 2 AND permission_name = 'all' AND permission_entity = 1"
            ),
            0
        );
    }

    #[test]
    fn test_update_entity() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let entity = EntityStruct {
            entity_id: Some(1),
            entity_name: "Organic chemistry".to_string(),
            entity_description: Some("Organic chemistry laboratory".to_string()),
            managers: Some(vec![person(3)]),
            ..Default::default()
        };
        assert_eq!(create_update_entity(&mut db, entity).unwrap(), 1);

        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM entity WHERE entity_id = 1 AND entity_name = 'Organic chemistry'"
            ),
            1
        );
        assert_eq!(manager_ids(&db, 1), [3]);
    }

    #[test]
    fn test_create_entity_with_invalid_manager() {
        let mut db = crate::test_utils::init_test_with_fixture();

        let entity = EntityStruct {
            entity_name: "Mathematics".to_string(),
            managers: Some(vec![person(999)]),
            ..Default::default()
        };

        assert!(create_update_entity(&mut db, entity).is_err());
        // Nothing is saved.
        assert_eq!(count(&db, "SELECT COUNT(*) FROM entity"), 3);
    }

    #[test]
    fn test_delete_entity() {
        let mut db = crate::test_utils::init_test_with_fixture();

        delete_entity(&mut db, 3).unwrap();

        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM entity WHERE entity_id = 3"),
            0
        );
    }

    #[test]
    fn test_delete_entity_with_store_locations() {
        let mut db = crate::test_utils::init_test_with_fixture();

        assert!(delete_entity(&mut db, 1).is_err());
        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM entity WHERE entity_id = 1"),
            1
        );
    }

    #[test]
    fn test_delete_entity_with_people() {
        let mut db = crate::test_utils::init_test_with_fixture();
        manage_physics(&mut db);

        // Physics has a member, and no store location: the member blocks the
        // deletion.
        let error = delete_entity(&mut db, 3).unwrap_err();
        assert!(
            error.to_string().contains("FOREIGN KEY constraint failed"),
            "{error}"
        );
        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM entity WHERE entity_id = 3"),
            1
        );
    }

    #[test]
    fn test_delete_entity_after_removing_members() {
        let mut db = crate::test_utils::init_test_with_fixture();
        manage_physics(&mut db);
        db.execute(
            "DELETE FROM personentities WHERE personentities_entity_id = 3",
            [],
        )
        .unwrap();

        delete_entity(&mut db, 3).unwrap();

        // The entity is gone, with its managers and permissions.
        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM entity WHERE entity_id = 3"),
            0
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM entitypeople WHERE entitypeople_entity_id = 3"
            ),
            0
        );
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM permission WHERE permission_entity = 3"
            ),
            0
        );
    }

    #[test]
    fn test_entity_name_exists() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(entity_name_exists(&db, "Biology", None).unwrap());
        assert!(entity_name_exists(&db, "Biology", Some(1)).unwrap());
        // The entity being updated keeps its own name.
        assert!(!entity_name_exists(&db, "Biology", Some(2)).unwrap());
        // Names are compared exactly, like the UNIQUE constraint.
        assert!(!entity_name_exists(&db, "biology", None).unwrap());
        assert!(!entity_name_exists(&db, "Unknown", None).unwrap());
    }

    #[test]
    fn test_entity_has_members_and_store_locations() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(entity_has_members(&db, 2).unwrap());
        assert!(!entity_has_members(&db, 3).unwrap());
        assert!(entity_has_store_locations(&db, 1).unwrap());
        assert!(!entity_has_store_locations(&db, 3).unwrap());
    }
}
