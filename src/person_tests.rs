#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::person::*;
    use rusqlite::Connection;

    fn init_test_person() -> Connection {
        let db = crate::test_utils::init_test();

        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Insert a few persons
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (100, 'test1@example.com')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO person (person_id, person_email) VALUES (101, 'test2@example.com')",
            [],
        )
        .unwrap();

        // Insert a few entities
        db.execute("INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (100, 'Test Entity 1', 'Desc 1')", []).unwrap();
        db.execute("INSERT INTO entity (entity_id, entity_name, entity_description) VALUES (101, 'Test Entity 2', 'Desc 2')", []).unwrap();

        db
    }

    #[test]
    fn test_set_person_manager() {
        let mut db_connection = init_test_person();

        // We use a transaction because set_person_manager usually takes &Transaction or &Connection
        // Depending on your signature, if it takes &Transaction:
        let tx = db_connection.transaction().unwrap();

        let person_id = 100;
        let entity_id = 100;

        set_person_manager(&tx, person_id, entity_id).expect("Should successfully set manager");
        tx.commit().unwrap();

        // 1. Verify the manager link exists in entitypeople
        let mut stmt = db_connection
            .prepare("SELECT count(*) FROM entitypeople WHERE entitypeople_person_id = ? AND entitypeople_entity_id = ?")
            .unwrap();
        let count: i32 = stmt
            .query_row([person_id, entity_id], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count, 1,
            "A manager link should have been created in entitypeople"
        );

        // 2. Verify the permissions were granted
        // Based on create_update_entity_managers, managers usually get 'all' item and 'all' name for that entity
        let mut stmt = db_connection
            .prepare("SELECT count(*) FROM permission WHERE person = ? AND permission_item = 'all' AND permission_name = 'all' AND permission_entity = ?")
            .unwrap();
        let perm_count: i32 = stmt
            .query_row([person_id, entity_id], |row| row.get(0))
            .unwrap();

        assert_eq!(
            perm_count, 1,
            "Manager should have been granted 'all' permissions for the entity"
        );
    }

    #[test]
    fn test_unset_person_manager() {
        let mut db_connection = init_test_person();

        let person_id = 100;
        let entity_id = 100;

        // Setup: First, make the person a manager
        {
            let tx = db_connection.transaction().unwrap();
            set_person_manager(&tx, person_id, entity_id).unwrap();
            tx.commit().unwrap();
        }

        // Verify setup was successful
        {
            let mut stmt = db_connection.prepare("SELECT count(*) FROM entitypeople WHERE entitypeople_person_id = ? AND entitypeople_entity_id = ?").unwrap();
            assert_eq!(
                stmt.query_row([person_id, entity_id], |row| row.get::<_, i32>(0))
                    .unwrap(),
                1
            );
        }

        // Now, unset the manager
        {
            let tx = db_connection.transaction().unwrap();
            unset_person_manager(&tx, person_id, entity_id)
                .expect("Should successfully unset manager");
            tx.commit().unwrap();
        }

        // 1. Verify the manager link is gone from entitypeople
        let mut stmt = db_connection
            .prepare("SELECT count(*) FROM entitypeople WHERE entitypeople_person_id = ? AND entitypeople_entity_id = ?")
            .unwrap();
        let count: i32 = stmt
            .query_row([person_id, entity_id], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count, 0,
            "The manager link should have been removed from entitypeople"
        );

        // 2. Verify the manager permissions are gone
        // We look for the 'all'/'all' permission specifically for that entity
        let mut stmt = db_connection
            .prepare("SELECT count(*) FROM permission WHERE person = ? AND permission_item = 'all' AND permission_name = 'all' AND permission_entity = ?")
            .unwrap();
        let perm_count: i32 = stmt
            .query_row([person_id, entity_id], |row| row.get(0))
            .unwrap();

        assert_eq!(
            perm_count, 0,
            "The manager permissions for the entity should have been removed"
        );
    }

    #[test]
    fn test_set_person_admin() {
        let mut conn = init_test_person();

        // Verify the person is not initially an admin
        let admins = get_admins(&conn).unwrap();
        assert!(admins.is_empty(), "No admins should exist initially");

        // Set the person as admin
        set_person_admin(&mut conn, 100).unwrap();

        // Verify the person is now an admin
        let admins = get_admins(&conn).unwrap();
        assert_eq!(admins.len(), 1, "Should have exactly one admin");
        assert_eq!(admins[0].person_id, Some(100));
        assert_eq!(admins[0].person_email, "test1@example.com");

        // Check if the person appears in get_people with is_admin flag
        let (people, _) = get_people(
            &conn,
            &chimitheque_types::requestfilter::RequestFilter::default(),
            100,
        )
        .unwrap();

        let admin_person = people.iter().find(|p| p.person_id == Some(100)).unwrap();
        assert!(admin_person.is_admin, "Person should be marked as admin");
    }

    #[test]
    fn test_unset_person_admin() {
        let mut conn = init_test_person();

        // Set the person as admin first
        set_person_admin(&mut conn, 100).unwrap();

        // Verify the person is now an admin
        let admins = get_admins(&conn).unwrap();
        assert_eq!(admins.len(), 1, "Person should be an admin after setup");
        assert_eq!(admins[0].person_id, Some(100));

        // Remove admin privileges
        unset_person_admin(&mut conn, 100).unwrap();

        // Verify the person is no longer an admin
        let admins = get_admins(&conn).unwrap();
        assert!(admins.is_empty(), "No admins should exist after unset");

        // Verify the permission record was removed
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM permission WHERE permission_item = 'all' AND permission_name = 'all' AND permission_entity IS NULL AND person = ?1",
                [100],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "Admin permission record should be removed");
    }

    #[test]
    fn test_delete_person() {
        let mut db_connection = init_test_person();

        // First, create a person to delete
        let person_id = 100;

        // Verify the person exists before deletion
        {
            let mut stmt = db_connection
                .prepare("SELECT count(*) FROM person WHERE person_id = ?")
                .unwrap();
            let count: i32 = stmt.query_row([person_id], |row| row.get(0)).unwrap();
            assert_eq!(count, 1, "Person should exist before deletion");
        }

        // Now delete the person
        {
            // let tx = db_connection.transaction().unwrap();
            delete_person(&mut db_connection, person_id)
                .expect("Should successfully delete person");
            // tx.commit().unwrap();
        }

        // Verify the person is gone
        {
            let mut stmt = db_connection
                .prepare("SELECT count(*) FROM person WHERE person_id = ?")
                .unwrap();
            let count: i32 = stmt.query_row([person_id], |row| row.get(0)).unwrap();
            assert_eq!(count, 0, "Person should be deleted");
        }

        // Also verify that any related records in entitypeople are gone
        {
            let mut stmt = db_connection
                .prepare("SELECT count(*) FROM entitypeople WHERE entitypeople_person_id = ?")
                .unwrap();
            let count: i32 = stmt.query_row([person_id], |row| row.get(0)).unwrap();
            assert_eq!(count, 0, "Related entitypeople records should be deleted");
        }

        // Also verify that any related permission records are gone
        {
            let mut stmt = db_connection
                .prepare("SELECT count(*) FROM permission WHERE person = ?")
                .unwrap();
            let count: i32 = stmt.query_row([person_id], |row| row.get(0)).unwrap();
            assert_eq!(count, 0, "Related permission records should be deleted");
        }
    }
}
