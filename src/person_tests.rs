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
}
