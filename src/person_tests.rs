#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::person::*;
    use chimitheque_types::entity::Entity;
    use rusqlite::Connection;

    // See the fixture description in resources/test_fixture.sql: admin (1),
    // chemistry.manager (2), chemistry.user (3), biology.manager (4),
    // biology.user (5), new.user (6) with no entity; Chemistry (1), Biology (2).

    fn count(db: &Connection, sql: &str, params: impl rusqlite::Params) -> i64 {
        db.query_row(sql, params, |row| row.get(0)).unwrap()
    }

    fn is_manager_row(db: &Connection, person_id: u64, entity_id: u64) -> bool {
        count(
            db,
            "SELECT COUNT(*) FROM entitypeople WHERE entitypeople_person_id = ?1 AND entitypeople_entity_id = ?2",
            [person_id, entity_id],
        ) == 1
    }

    fn has_manager_permission(db: &Connection, person_id: u64, entity_id: u64) -> bool {
        count(
            db,
            "SELECT COUNT(*) FROM permission WHERE person = ?1 AND permission_name = 'all' AND permission_item = 'all' AND permission_entity = ?2",
            [person_id, entity_id],
        ) == 1
    }

    // The ids of the people the person can see, sorted.
    fn visible_people(db: &Connection, person_id: u64) -> Vec<u64> {
        let (people, _) = get_people(db, &RequestFilter::default(), person_id).unwrap();
        let mut ids: Vec<u64> = people
            .iter()
            .filter_map(|person| person.person_id)
            .collect();
        ids.sort_unstable();
        ids
    }

    fn admin_ids(db: &Connection) -> Vec<u64> {
        let mut ids: Vec<u64> = get_admins(db)
            .unwrap()
            .iter()
            .filter_map(|person| person.person_id)
            .collect();
        ids.sort_unstable();
        ids
    }

    #[test]
    fn test_set_person_manager() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.user becomes a manager of Chemistry.
        let tx = db.transaction().unwrap();
        set_person_manager(&tx, 3, 1).unwrap();
        tx.commit().unwrap();

        assert!(is_manager_row(&db, 3, 1));
        assert!(has_manager_permission(&db, 3, 1));
    }

    #[test]
    fn test_unset_person_manager() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.manager is no longer a manager of Chemistry.
        let tx = db.transaction().unwrap();
        unset_person_manager(&tx, 2, 1).unwrap();
        tx.commit().unwrap();

        assert!(!is_manager_row(&db, 2, 1));
        assert!(!has_manager_permission(&db, 2, 1));
        // They are still a member.
        assert!(is_person_member_of(&db, 2, 1).unwrap());
    }

    #[test]
    fn test_set_and_unset_person_admin() {
        let mut db = crate::test_utils::init_test_with_fixture();
        assert_eq!(admin_ids(&db), [1]);

        set_person_admin(&mut db, 3).unwrap();
        assert_eq!(admin_ids(&db), [1, 3]);
        assert!(is_person_admin(&db, 3).unwrap());
        let (people, _) = get_people(&db, &RequestFilter::default(), 3).unwrap();
        let person = people
            .iter()
            .find(|person| person.person_id == Some(3))
            .unwrap();
        assert!(person.is_admin);

        unset_person_admin(&mut db, 3).unwrap();
        assert_eq!(admin_ids(&db), [1]);
        assert!(!is_person_admin(&db, 3).unwrap());
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM permission WHERE person = 3 AND permission_name = 'all' AND permission_item = 'all' AND permission_entity IS NULL",
                []
            ),
            0
        );
    }

    #[test]
    fn test_delete_person() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.user has storages, a borrowing, a bookmark, a membership
        // and permissions.
        delete_person(&mut db, 3).unwrap();

        assert_eq!(
            count(&db, "SELECT COUNT(*) FROM person WHERE person_id = 3", []),
            0
        );
        for table_and_column in [
            "permission WHERE person",
            "personentities WHERE personentities_person_id",
            "bookmark WHERE person",
            "borrowing WHERE borrower",
            "storage WHERE person",
        ] {
            assert_eq!(
                count(
                    &db,
                    &format!("SELECT COUNT(*) FROM {table_and_column} = 3"),
                    []
                ),
                0,
                "{table_and_column}"
            );
        }
        // Their storages are kept, saved by the admin.
        assert_eq!(
            count(
                &db,
                "SELECT COUNT(*) FROM storage WHERE storage_id IN (1, 2, 3, 4) AND person = 1",
                []
            ),
            4
        );
    }

    #[test]
    fn test_manager_cannot_get_people_from_other_entities() {
        let db = crate::test_utils::init_test_with_fixture();

        // A manager sees the people of their entity, and the people with no
        // entity: the admin and new.user.
        assert_eq!(visible_people(&db, 2), [1, 2, 3, 6]);
        assert_eq!(visible_people(&db, 4), [1, 4, 5, 6]);
    }

    #[test]
    fn test_member_cannot_get_people_from_other_entities() {
        let db = crate::test_utils::init_test_with_fixture();

        // chemistry.user reads the people of Chemistry.
        assert_eq!(visible_people(&db, 3), [1, 2, 3, 6]);
    }

    #[test]
    fn test_person_with_no_entity_can_only_get_people_with_no_membership() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(visible_people(&db, 6), [1, 6]);
    }

    #[test]
    fn test_admin_gets_all_the_people() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(visible_people(&db, 1), [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_is_person_manager() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(is_person_manager(&db, 2, 1).unwrap());
        assert!(is_person_manager(&db, 4, 2).unwrap());
        // A member, a person with no entity, a manager of another entity.
        assert!(!is_person_manager(&db, 3, 1).unwrap());
        assert!(!is_person_manager(&db, 6, 1).unwrap());
        assert!(!is_person_manager(&db, 2, 2).unwrap());
    }

    #[test]
    fn test_is_person_member_of() {
        let db = crate::test_utils::init_test_with_fixture();

        for (person_id, entity_id) in [(2, 1), (3, 1), (4, 2), (5, 2)] {
            assert!(is_person_member_of(&db, person_id, entity_id).unwrap());
        }
        assert!(!is_person_member_of(&db, 3, 2).unwrap());
        assert!(!is_person_member_of(&db, 6, 1).unwrap());
    }

    #[test]
    fn test_person_email_exists() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(person_email_exists(&db, "chemistry.user@example.com", None).unwrap());
        assert!(person_email_exists(&db, "chemistry.user@example.com", Some(2)).unwrap());
        // The person being updated keeps their own email.
        assert!(!person_email_exists(&db, "chemistry.user@example.com", Some(3)).unwrap());
        assert!(!person_email_exists(&db, "unknown@example.com", None).unwrap());
    }

    #[test]
    fn test_managed_and_member_entity_ids() {
        let db = crate::test_utils::init_test_with_fixture();

        assert_eq!(managed_entity_ids(&db, 2).unwrap(), [1]);
        assert_eq!(managed_entity_ids(&db, 3).unwrap(), [0u64; 0]);
        assert_eq!(member_entity_ids(&db, 3).unwrap(), [1]);
        assert_eq!(member_entity_ids(&db, 6).unwrap(), [0u64; 0]);
    }

    #[test]
    fn test_person_permissions() {
        let db = crate::test_utils::init_test_with_fixture();

        // new.user only reads the products.
        let permissions = person_permissions(&db, 6).unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].permission_name, PermissionName::Read);
        assert_eq!(permissions[0].permission_item, PermissionItem::Products);
        assert_eq!(permissions[0].permission_entity, None);

        assert_eq!(person_permissions(&db, 3).unwrap().len(), 3);
    }

    #[test]
    fn test_create_update_person_keeps_the_stored_managers() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // chemistry.manager manages Chemistry (1); the saved person claims
        // Biology (2) instead.
        let person = PersonStruct {
            person_id: Some(2),
            person_email: "chemistry.manager@example.com".to_string(),
            entities: Some(vec![Entity {
                entity_id: Some(1),
                ..Default::default()
            }]),
            managed_entities: Some(vec![Entity {
                entity_id: Some(2),
                ..Default::default()
            }]),
            ..Default::default()
        };
        create_update_person(&mut db, person).unwrap();

        assert!(has_manager_permission(&db, 2, 1));
        assert!(!has_manager_permission(&db, 2, 2));
        assert_eq!(managed_entity_ids(&db, 2).unwrap(), [1]);
    }
}
