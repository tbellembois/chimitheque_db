#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::person::*;
    use crate::test_utils::{init_test_with_fixture, profiles::*};
    use chimitheque_types::entity::Entity;
    use rusqlite::Connection;

    // See the fixture description in resources/test_fixture.sql. The people
    // with no entity: the admin, new.user, all.reader and product.editor.
    const NO_ENTITY: [u64; 4] = [ADMIN, NEW_USER, ALL_READER, PRODUCT_EDITOR];

    fn count(db: &Connection, sql: &str, params: impl rusqlite::Params) -> i64 {
        db.query_row(sql, params, |row| row.get(0)).unwrap()
    }

    fn has_manager_permission(db: &Connection, person_id: u64, entity_id: u64) -> bool {
        count(
            db,
            "SELECT COUNT(*) FROM permission WHERE person = ?1 AND permission_name = 'all' AND permission_item = 'all' AND permission_entity = ?2",
            [person_id, entity_id],
        ) == 1
    }

    // The sorted ids of the people the person gets with the filter.
    fn people_ids(db: &Connection, filter: &RequestFilter, person_id: u64) -> Vec<u64> {
        let (people, count) = get_people(db, filter, person_id).unwrap();
        assert_eq!(count, people.len());
        let mut ids: Vec<u64> = people
            .iter()
            .filter_map(|person| person.person_id)
            .collect();
        ids.sort_unstable();
        ids
    }

    fn sorted(mut ids: Vec<u64>) -> Vec<u64> {
        ids.sort_unstable();
        ids
    }

    fn entity(entity_id: u64) -> Entity {
        Entity {
            entity_id: Some(entity_id),
            ..Default::default()
        }
    }

    #[test]
    fn test_get_people_permissions() {
        let db = init_test_with_fixture();
        let chemistry = sorted([&[CHEMISTRY_MANAGER, CHEMISTRY_USER][..], &NO_ENTITY].concat());
        let biology = sorted([&[BIOLOGY_MANAGER, BIOLOGY_USER][..], &NO_ENTITY].concat());

        // A permission on the entities gives the people of those entities;
        // the people with no entity are seen by everyone.
        for (person_id, expected) in [
            (ADMIN, (1..=8).collect()),
            (CHEMISTRY_MANAGER, chemistry.clone()),
            (CHEMISTRY_USER, chemistry),
            (BIOLOGY_MANAGER, biology.clone()),
            (BIOLOGY_USER, biology),
            (NEW_USER, NO_ENTITY.to_vec()),
            (ALL_READER, NO_ENTITY.to_vec()),
            (PRODUCT_EDITOR, NO_ENTITY.to_vec()),
        ] {
            assert_eq!(
                people_ids(&db, &RequestFilter::default(), person_id),
                expected,
                "{person_id}"
            );
        }
    }

    #[test]
    fn test_get_people_filters() {
        let db = init_test_with_fixture();

        let filter = RequestFilter {
            entity: Some(1),
            ..Default::default()
        };
        assert_eq!(
            people_ids(&db, &filter, ADMIN),
            [CHEMISTRY_MANAGER, CHEMISTRY_USER]
        );
        // Not more than what the person can see.
        assert_eq!(people_ids(&db, &filter, BIOLOGY_MANAGER), [0u64; 0]);

        let filter = RequestFilter {
            search: Some("biology".to_string()),
            ..Default::default()
        };
        assert_eq!(
            people_ids(&db, &filter, ADMIN),
            [BIOLOGY_MANAGER, BIOLOGY_USER]
        );

        let filter = RequestFilter {
            person_email: Some("new.user@example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(people_ids(&db, &filter, ADMIN), [NEW_USER]);
    }

    #[test]
    fn test_get_people_loads_the_person() {
        let db = init_test_with_fixture();
        let filter = RequestFilter {
            id: Some(CHEMISTRY_MANAGER),
            ..Default::default()
        };

        let (people, _) = get_people(&db, &filter, ADMIN).unwrap();
        let person = &people[0];

        assert_eq!(person.person_email, "chemistry.manager@example.com");
        assert!(!person.is_admin);
        let entity_ids = |entities: &Option<Vec<Entity>>| -> Vec<u64> {
            entities
                .iter()
                .flatten()
                .filter_map(|entity| entity.entity_id)
                .collect()
        };
        assert_eq!(entity_ids(&person.entities), [1]);
        assert_eq!(entity_ids(&person.managed_entities), [1]);
        assert_eq!(person.permissions.as_ref().unwrap().len(), 3);

        let filter = RequestFilter {
            id: Some(ADMIN),
            ..Default::default()
        };
        let (people, _) = get_people(&db, &filter, ADMIN).unwrap();
        assert!(people[0].is_admin);
        assert!(people[0].entities.is_none());
    }

    #[test]
    fn test_create_person() {
        let mut db = init_test_with_fixture();

        // A member of Biology who writes its storages.
        let person = PersonStruct {
            person_email: "biology.technician@example.com".to_string(),
            entities: Some(vec![entity(2)]),
            permissions: Some(vec![PermissionStruct {
                permission_name: PermissionName::Write,
                permission_item: PermissionItem::Storages,
                permission_entity: Some(2),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let person_id = create_update_person(&mut db, person).unwrap();

        assert_eq!(person_id, 9);
        assert_eq!(member_entity_ids(&db, 9).unwrap(), [2]);
        // The given permission, plus reading the products and the entity.
        let mut permissions: Vec<(String, String, Option<u64>)> = person_permissions(&db, 9)
            .unwrap()
            .into_iter()
            .map(|permission| {
                (
                    permission.permission_name.to_string(),
                    permission.permission_item.to_string(),
                    permission.permission_entity,
                )
            })
            .collect();
        permissions.sort();
        assert_eq!(
            permissions,
            [
                ("r".to_string(), "entities".to_string(), Some(2)),
                ("r".to_string(), "products".to_string(), None),
                ("w".to_string(), "storages".to_string(), Some(2)),
            ]
        );
    }

    #[test]
    fn test_update_person_replaces_permissions_and_memberships() {
        let mut db = init_test_with_fixture();

        // chemistry.user moved to Biology, reading its storages.
        let person = PersonStruct {
            person_id: Some(CHEMISTRY_USER),
            person_email: "chemistry.user@example.com".to_string(),
            entities: Some(vec![entity(2)]),
            permissions: Some(vec![PermissionStruct {
                permission_name: PermissionName::Read,
                permission_item: PermissionItem::Storages,
                permission_entity: Some(2),
                ..Default::default()
            }]),
            ..Default::default()
        };
        create_update_person(&mut db, person).unwrap();

        assert_eq!(member_entity_ids(&db, CHEMISTRY_USER).unwrap(), [2]);
        let permissions = person_permissions(&db, CHEMISTRY_USER).unwrap();
        assert_eq!(permissions.len(), 3);
        assert!(
            !permissions
                .iter()
                .any(|permission| permission.permission_entity == Some(1))
        );
    }

    #[test]
    fn test_create_update_person_keeps_the_stored_managers() {
        let mut db = init_test_with_fixture();

        // chemistry.manager manages Chemistry (1); the saved person claims
        // Biology (2) instead.
        let person = PersonStruct {
            person_id: Some(CHEMISTRY_MANAGER),
            person_email: "chemistry.manager@example.com".to_string(),
            entities: Some(vec![entity(1)]),
            managed_entities: Some(vec![entity(2)]),
            ..Default::default()
        };
        create_update_person(&mut db, person).unwrap();

        assert!(has_manager_permission(&db, CHEMISTRY_MANAGER, 1));
        assert!(!has_manager_permission(&db, CHEMISTRY_MANAGER, 2));
        assert_eq!(managed_entity_ids(&db, CHEMISTRY_MANAGER).unwrap(), [1]);
    }

    #[test]
    fn test_set_and_unset_person_manager() {
        let mut db = init_test_with_fixture();

        // chemistry.user becomes a manager of Chemistry.
        let tx = db.transaction().unwrap();
        set_person_manager(&tx, CHEMISTRY_USER, 1).unwrap();
        tx.commit().unwrap();
        assert!(is_person_manager(&db, CHEMISTRY_USER, 1).unwrap());
        assert!(has_manager_permission(&db, CHEMISTRY_USER, 1));

        // new.user, with no entity, also becomes a member.
        let tx = db.transaction().unwrap();
        set_person_manager(&tx, NEW_USER, 2).unwrap();
        tx.commit().unwrap();
        assert!(is_person_member_of(&db, NEW_USER, 2).unwrap());

        // chemistry.manager is no longer a manager, but still a member.
        let tx = db.transaction().unwrap();
        unset_person_manager(&tx, CHEMISTRY_MANAGER, 1).unwrap();
        tx.commit().unwrap();
        assert!(!is_person_manager(&db, CHEMISTRY_MANAGER, 1).unwrap());
        assert!(!has_manager_permission(&db, CHEMISTRY_MANAGER, 1));
        assert!(is_person_member_of(&db, CHEMISTRY_MANAGER, 1).unwrap());
    }

    #[test]
    fn test_set_and_unset_person_admin() {
        let mut db = init_test_with_fixture();
        let admin_ids = |db: &Connection| -> Vec<u64> {
            sorted(
                get_admins(db)
                    .unwrap()
                    .iter()
                    .filter_map(|person| person.person_id)
                    .collect(),
            )
        };
        assert_eq!(admin_ids(&db), [ADMIN]);

        set_person_admin(&mut db, CHEMISTRY_USER).unwrap();
        assert_eq!(admin_ids(&db), [ADMIN, CHEMISTRY_USER]);
        assert!(is_person_admin(&db, CHEMISTRY_USER).unwrap());
        // As an admin, they see all the people.
        assert_eq!(
            people_ids(&db, &RequestFilter::default(), CHEMISTRY_USER).len(),
            8
        );

        unset_person_admin(&mut db, CHEMISTRY_USER).unwrap();
        assert_eq!(admin_ids(&db), [ADMIN]);
        assert!(!is_person_admin(&db, CHEMISTRY_USER).unwrap());
    }

    #[test]
    fn test_delete_person() {
        let mut db = init_test_with_fixture();

        // chemistry.user has storages, a borrowing, a bookmark, a membership
        // and permissions.
        delete_person(&mut db, CHEMISTRY_USER).unwrap();

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
    fn test_admin_manager_and_member_lookups() {
        let db = init_test_with_fixture();

        assert!(is_person_admin(&db, ADMIN).unwrap());
        assert!(!is_person_admin(&db, CHEMISTRY_MANAGER).unwrap());

        assert!(is_person_manager(&db, CHEMISTRY_MANAGER, 1).unwrap());
        assert!(!is_person_manager(&db, CHEMISTRY_MANAGER, 2).unwrap());
        assert!(!is_person_manager(&db, CHEMISTRY_USER, 1).unwrap());
        assert_eq!(managed_entity_ids(&db, BIOLOGY_MANAGER).unwrap(), [2]);
        assert_eq!(managed_entity_ids(&db, BIOLOGY_USER).unwrap(), [0u64; 0]);

        assert!(is_person_member_of(&db, CHEMISTRY_USER, 1).unwrap());
        assert!(!is_person_member_of(&db, CHEMISTRY_USER, 2).unwrap());
        assert!(!is_person_member_of(&db, ALL_READER, 1).unwrap());
        assert_eq!(member_entity_ids(&db, BIOLOGY_USER).unwrap(), [2]);
        assert_eq!(member_entity_ids(&db, NEW_USER).unwrap(), [0u64; 0]);
    }

    #[test]
    fn test_person_permissions() {
        let db = init_test_with_fixture();

        // new.user only reads the products.
        let permissions = person_permissions(&db, NEW_USER).unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].permission_name, PermissionName::Read);
        assert_eq!(permissions[0].permission_item, PermissionItem::Products);
        assert_eq!(permissions[0].permission_entity, None);

        assert_eq!(person_permissions(&db, ALL_READER).unwrap().len(), 2);
        assert_eq!(person_permissions(&db, 999).unwrap().len(), 0);
    }

    #[test]
    fn test_person_email_exists() {
        let db = init_test_with_fixture();

        assert!(person_email_exists(&db, "chemistry.user@example.com", None).unwrap());
        assert!(
            person_email_exists(&db, "chemistry.user@example.com", Some(CHEMISTRY_MANAGER))
                .unwrap()
        );
        // The person being updated keeps their own email.
        assert!(
            !person_email_exists(&db, "chemistry.user@example.com", Some(CHEMISTRY_USER)).unwrap()
        );
        assert!(!person_email_exists(&db, "unknown@example.com", None).unwrap());
    }
}
