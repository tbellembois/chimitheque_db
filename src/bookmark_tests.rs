#[cfg(test)]
mod tests {

    use crate::bookmark::*;
    use rusqlite::Connection;

    fn init_test_bookmarks() -> Connection {
        let db = crate::test_utils::init_test();

        // Disable synchronous operations and foreign key constraints for faster test execution
        db.execute("PRAGMA synchronous = OFF", []).unwrap();
        db.execute("PRAGMA foreign_keys = OFF", []).unwrap();

        // Delete existing records if any
        db.execute("DELETE FROM bookmark", []).unwrap();

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

        // Insert example data into the bookmark table
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (1, 1, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (2, 1, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (3, 2, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (4, 2, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (5, 3, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (6, 3, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (7, 4, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (8, 4, 2)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (9, 5, 1)",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO bookmark (bookmark_id, person, product) VALUES (10, 5, 2)",
            [],
        )
        .unwrap();

        // Enable foreign key constraints back
        db.execute("PRAGMA foreign_keys = ON", []).unwrap();

        db
    }

    #[test]
    fn test_toggle_product_bookmark_add_bookmark() {
        let mut db = init_test_bookmarks();
        let person_id = 4;
        let product_id = 3;

        // Call the function to toggle bookmark
        toggle_product_bookmark(&mut db, person_id, product_id).unwrap();

        // Verify the bookmark was added
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM bookmark WHERE person = ? AND product = ?",
                [person_id, product_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_toggle_product_bookmark_remove_bookmark() {
        let mut db = init_test_bookmarks();
        let person_id = 1;
        let product_id = 1;

        // Call the function to toggle bookmark (should remove existing bookmark)
        toggle_product_bookmark(&mut db, person_id, product_id).unwrap();

        // Verify the bookmark was removed
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM bookmark WHERE person = ? AND product = ?",
                [person_id, product_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_toggle_product_bookmark_no_change() {
        let mut db = init_test_bookmarks();
        let person_id = 1;
        let product_id = 3; // Product that wasn't bookmarked by this person

        // Call the function to toggle bookmark (should add bookmark)
        toggle_product_bookmark(&mut db, person_id, product_id).unwrap();

        // Call the function again (should remove the bookmark)
        toggle_product_bookmark(&mut db, person_id, product_id).unwrap();

        // Verify the bookmark was removed
        let count = db
            .query_row(
                "SELECT COUNT(*) FROM bookmark WHERE person = ? AND product = ?",
                [person_id, product_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }
}
