#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::borrowing::*;
    use rusqlite::Connection;

    fn init_test_borrowings() -> Connection {
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

        // Insert example data into the borrowing table
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (1, 'comment 1', 1, 1, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (2, 'comment 2', 2, 1, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (3, 'comment 3', 1, 1, 3)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (4, 'comment 4', 2, 2, 4)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (5, 'comment 5', 1, 1, 5)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (6, 'comment 6', 3, 3, 6)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (7, 'comment 7', 1, 1, 7)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (8, 'comment 8', 2, 2, 8)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO borrowing (borrowing_id, borrowing_comment, person, borrower, storage) VALUES (9, 'comment 9', 1, 1, 9)",
            [],
        )
        .unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_toggle_product_borrowing_add_borrowing() {
        let mut db = init_test_borrowings();
        let person_id = 4;
        let borrower_id = 4;
        let storage_id = 10;
        let borrowing_comment = Some("foo".to_string());

        // Call the function to toggle borrowing
        toggle_storage_borrowing(
            &mut db,
            person_id,
            storage_id,
            borrower_id,
            borrowing_comment,
        )
        .unwrap();

        // Verify the borrowing was added
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM borrowing WHERE person = ? AND storage = ?",
                [person_id, storage_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_toggle_product_borrowing_remove_borrowing() {
        let mut db = init_test_borrowings();
        let person_id = 2;
        let borrower_id = 2;
        let storage_id = 8;
        let borrowing_comment = None;

        // Call the function to toggle borrowing (should remove existing borrowing)
        toggle_storage_borrowing(
            &mut db,
            person_id,
            storage_id,
            borrower_id,
            borrowing_comment,
        )
        .unwrap();

        // Verify the borrowing was removed
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM borrowing WHERE person = ? AND storage = ?",
                [person_id, storage_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_toggle_product_borrowing_no_change() {
        let mut db = init_test_borrowings();
        let person_id = 2;
        let borrower_id = 1;
        let storage_id = 10;
        let borrowing_comment = None;

        // Call the function to toggle borrowing (should add borrowing)
        toggle_storage_borrowing(
            &mut db,
            person_id,
            storage_id,
            borrower_id,
            borrowing_comment.clone(),
        )
        .unwrap();

        // Call the function again (should remove the borrowing)
        toggle_storage_borrowing(
            &mut db,
            person_id,
            storage_id,
            borrower_id,
            borrowing_comment,
        )
        .unwrap();

        // Verify the borrowing was removed
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM borrowing WHERE person = ? AND storage = ?",
                [person_id, storage_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }
}
