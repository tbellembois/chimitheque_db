#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::{
        init::populate_db_with_base_data,
        product::{create_update_product, get_products},
        pubchemproduct::*,
    };
    use chimitheque_types::{
        person::Person, pubchemproduct::PubchemProduct, requestfilter::RequestFilter,
    };

    fn init_test_pubchemproduct() -> Connection {
        let mut db = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db).unwrap();
        db
    }

    fn ethanol() -> PubchemProduct {
        PubchemProduct {
            name: Some("ethanol".to_string()),
            cas: Some("64-17-5".to_string()),
            molecular_formula: Some("C2H6O".to_string()),
            molecular_weight: Some("46.07".to_string()),
            molecular_weight_unit: Some("g/mol".to_string()),
            signal: Some(vec!["Danger".to_string()]),
            symbols: Some(vec!["GHS02".to_string(), "GHS07".to_string()]),
            hs: Some(vec!["H225".to_string(), "H319".to_string()]),
            ps: Some(vec!["P210".to_string(), "P233".to_string()]),
            ..Default::default()
        }
    }

    /// Imports `pubchem_product` as the handler does, onto `product_id` if set.
    fn import(
        db_connection: &mut Connection,
        pubchem_product: &PubchemProduct,
        product_id: Option<u64>,
    ) -> u64 {
        let mut product = pubchem_product.to_product().unwrap();
        resolve_pick_lists(db_connection, &mut product).unwrap();
        product.person = Person {
            person_id: Some(1),
            ..Default::default()
        };
        product.product_id = product_id;
        product.sanitize_and_validate().unwrap();
        create_update_product(db_connection, product).unwrap()
    }

    #[test]
    fn test_resolve_pick_lists_combined_and_suffixed_codes() {
        let db_connection = init_test_pubchemproduct();
        let pubchem_product = PubchemProduct {
            name: Some("acetic acid".to_string()),
            hs: Some(vec!["H350i".to_string(), "H300+H310+H330".to_string()]),
            ps: Some(vec![
                "P301+P330+P331".to_string(),
                "P370+P380+P375[+P378]".to_string(),
            ]),
            ..Default::default()
        };
        let mut product = pubchem_product.to_product().unwrap();

        resolve_pick_lists(&db_connection, &mut product).unwrap();

        let hazard_statements = product.hazard_statements.unwrap();
        assert_eq!(
            hazard_statements[1].hazard_statement_label,
            "Fatal if swallowed, in contact with skin or if inhaled"
        );
        let precautionary_statements = product.precautionary_statements.unwrap();
        assert_eq!(
            precautionary_statements[0].precautionary_statement_label,
            "IF SWALLOWED: Rinse mouth. Do NOT induce vomiting."
        );
        assert!(
            precautionary_statements
                .iter()
                .all(|statement| statement.precautionary_statement_id.is_some())
        );
    }

    #[test]
    fn test_resolve_pick_lists() {
        let db_connection = init_test_pubchemproduct();
        let mut product = ethanol().to_product().unwrap();

        resolve_pick_lists(&db_connection, &mut product).unwrap();

        let symbols = product.symbols.as_ref().unwrap();
        assert!(symbols.iter().all(|symbol| symbol.symbol_id.is_some()));
        let signal_word = product.signal_word.as_ref().unwrap();
        assert!(signal_word.signal_word_id.is_some());
        // Matched ignoring case, replaced by the stored label.
        assert_eq!(signal_word.signal_word_label, "danger");
        let hazard_statements = product.hazard_statements.as_ref().unwrap();
        assert!(
            hazard_statements
                .iter()
                .all(|statement| statement.hazard_statement_id.is_some()
                    && !statement.hazard_statement_label.is_empty())
        );
        let precautionary_statements = product.precautionary_statements.as_ref().unwrap();
        assert!(
            precautionary_statements
                .iter()
                .all(|statement| statement.precautionary_statement_id.is_some())
        );
        assert_eq!(
            product.unit_molecular_weight.as_ref().unwrap().unit_id,
            Some(24)
        );
    }

    #[test]
    fn test_resolve_pick_lists_rejects_unknown_labels() {
        let db_connection = init_test_pubchemproduct();
        let cases = [
            (
                PubchemProduct {
                    symbols: Some(vec!["GHS10".to_string()]),
                    ..Default::default()
                },
                ImportPubchemProductError::UnknownSymbol("GHS10".to_string()),
            ),
            (
                PubchemProduct {
                    hs: Some(vec!["H999".to_string()]),
                    ..Default::default()
                },
                ImportPubchemProductError::UnknownHazardstatement("H999".to_string()),
            ),
            (
                PubchemProduct {
                    ps: Some(vec!["P999".to_string()]),
                    ..Default::default()
                },
                ImportPubchemProductError::UnknownPrecautionarystatement("P999".to_string()),
            ),
            (
                PubchemProduct {
                    signal: Some(vec!["Beware".to_string()]),
                    ..Default::default()
                },
                ImportPubchemProductError::UnknownSignalword("Beware".to_string()),
            ),
            (
                PubchemProduct {
                    molecular_weight_unit: Some("lb".to_string()),
                    ..Default::default()
                },
                ImportPubchemProductError::UnknownMolecularWeightUnit("lb".to_string()),
            ),
        ];
        for (pubchem_product, expected) in cases {
            let mut product = pubchem_product.to_product().unwrap();
            let error = resolve_pick_lists(&db_connection, &mut product).unwrap_err();
            assert_eq!(
                error.downcast_ref::<ImportPubchemProductError>(),
                Some(&expected)
            );
        }
    }

    #[test]
    fn test_import_creates_product() {
        let mut db_connection = init_test_pubchemproduct();

        let product_id = import(&mut db_connection, &ethanol(), None);

        let filter = RequestFilter {
            id: Some(product_id),
            ..Default::default()
        };
        let (products, count) = get_products(&db_connection, filter, 1).unwrap();
        assert_eq!(count, 1);
        let product = &products[0];
        assert_eq!(product.name.name_label, "ETHANOL");
        assert_eq!(
            product.cas_number.as_ref().unwrap().cas_number_label,
            "64-17-5"
        );
        assert_eq!(
            product
                .empirical_formula
                .as_ref()
                .unwrap()
                .empirical_formula_label,
            "C2H6O"
        );
        assert_eq!(product.product_molecular_weight, Some(46.07));
        assert_eq!(product.symbols.as_ref().unwrap().len(), 2);
        assert_eq!(product.hazard_statements.as_ref().unwrap().len(), 2);
        assert_eq!(product.precautionary_statements.as_ref().unwrap().len(), 2);

        // Importing the same product again reuses the name and CAS number.
        let second_id = import(&mut db_connection, &ethanol(), None);
        assert_ne!(second_id, product_id);
        let names: i64 = db_connection
            .query_row(
                "SELECT COUNT(*) FROM name WHERE name_label = 'ETHANOL'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(names, 1);
    }

    #[test]
    fn test_reimport_replaces_product_and_keeps_bookmarks() {
        let mut db_connection = init_test_pubchemproduct();
        let product_id = import(&mut db_connection, &ethanol(), None);
        db_connection
            .execute(
                "INSERT INTO bookmark (person, product) VALUES (1, ?1)",
                [product_id],
            )
            .unwrap();

        // Re-import with fewer data onto the same product.
        let reimported = PubchemProduct {
            name: Some("ethanol".to_string()),
            symbols: Some(vec!["GHS02".to_string()]),
            ..Default::default()
        };
        assert_eq!(
            import(&mut db_connection, &reimported, Some(product_id)),
            product_id
        );

        let filter = RequestFilter {
            id: Some(product_id),
            ..Default::default()
        };
        let (products, _) = get_products(&db_connection, filter, 1).unwrap();
        let product = &products[0];
        // Fields and links are replaced by the imported data.
        assert!(product.cas_number.is_none());
        assert!(product.product_molecular_weight.is_none());
        assert_eq!(product.symbols.as_ref().unwrap().len(), 1);
        assert!(product.hazard_statements.is_none());
        // Bookmarks are not product data: they are kept.
        let bookmarks: i64 = db_connection
            .query_row(
                "SELECT COUNT(*) FROM bookmark WHERE product = ?1",
                [product_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(bookmarks, 1);
    }
}
