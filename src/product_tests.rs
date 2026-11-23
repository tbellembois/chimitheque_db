#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use crate::product::*;
    use crate::test_utils::profiles::*;
    use chimitheque_types::{
        casnumber::CasNumber as CasNumberStruct, category::Category as CategoryStruct,
        classofcompound::ClassOfCompound as ClassOfCompoundStruct,
        empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct,
        hazardstatement::HazardStatement as HazardStatementStruct, name::Name as NameStruct,
        person::Person as PersonStruct,
        precautionarystatement::PrecautionaryStatement as PrecautionaryStatementStruct,
        product::Product as ProductStruct, producttype::ProductType, requestfilter::RequestFilter,
        signalword::SignalWord as SignalWordStruct, supplier::Supplier as SupplierStruct,
        supplierref::SupplierRef as SupplierRefStruct, symbol::Symbol as SymbolStruct,
        tag::Tag as TagStruct, unittype::UnitType,
    };
    use rusqlite::Connection;

    // See the fixture description in resources/test_fixture.sql: 1 Water,
    // 2 Ethanol (with all the fields and links), 3 Sodium chloride (no
    // storage), 4 Acetone (restricted), 5 Bovine serum albumin (bio), 6 Pipette
    // tips (consumable, no storage).
    const ETHANOL: u64 = 2;

    // The id of a base data row by its label.
    fn id_of(db: &Connection, table: &str, column: &str, label: &str) -> u64 {
        db.query_row(
            &format!("SELECT {table}_id FROM {table} WHERE {column} = ?1"),
            [label],
            |row| row.get(0),
        )
        .unwrap()
    }

    // The sorted ids of the products the person gets with the filter.
    fn product_ids(db: &Connection, filter: RequestFilter, person_id: u64) -> Vec<u64> {
        let (products, count) = get_products(db, filter, person_id).unwrap();
        assert_eq!(count, products.len());
        let mut ids: Vec<u64> = products
            .iter()
            .filter_map(|product| product.product_id)
            .collect();
        ids.sort_unstable();
        ids
    }

    fn get_product(db: &Connection, product_id: u64) -> ProductStruct {
        let filter = RequestFilter {
            id: Some(product_id),
            ..Default::default()
        };
        get_products(db, filter, 1).unwrap().0.remove(0)
    }

    /// Labels of the populated items, sorted: the populate queries have no
    /// ORDER BY, so the item order is not guaranteed.
    fn sorted_labels<T>(items: &[T], label: impl Fn(&T) -> &String) -> Vec<String> {
        let mut labels: Vec<String> = items.iter().map(|item| label(item).clone()).collect();
        labels.sort();
        labels
    }

    #[test]
    fn test_get_products_permissions() {
        let db = crate::test_utils::init_test_with_fixture();
        let all = vec![1, 2, 3, 4, 5, 6];
        let not_restricted = vec![1, 2, 3, 5, 6];

        // Everyone reads the products (they do not belong to an entity); the
        // restricted Acetone (4) needs r on rproducts, writing the products,
        // or the all item.
        for (person_id, expected) in [
            (ADMIN, all.clone()),
            (CHEMISTRY_MANAGER, all.clone()),
            (CHEMISTRY_USER, not_restricted.clone()),
            (BIOLOGY_MANAGER, all.clone()),
            (BIOLOGY_USER, all.clone()),
            (NEW_USER, not_restricted.clone()),
            (ALL_READER, not_restricted),
            (PRODUCT_EDITOR, all),
        ] {
            assert_eq!(
                product_ids(&db, RequestFilter::default(), person_id),
                expected,
                "{person_id}"
            );
        }

        // Without a products permission, no product.
        db.execute("DELETE FROM permission WHERE person = ?1", [NEW_USER])
            .unwrap();
        assert_eq!(
            product_ids(&db, RequestFilter::default(), NEW_USER),
            [0u64; 0]
        );
    }

    #[test]
    fn test_get_products_storage_counts_per_profile() {
        let db = crate::test_utils::init_test_with_fixture();

        // Ethanol storages: 2 (Chemistry), 3 (Chemistry, archived), 7
        // (Biology), and the history row 4. The counts are those of the
        // entities the person reads the storages of; the total is for all.
        for (person_id, current, archived) in [
            (ADMIN, 2, 1),
            (CHEMISTRY_MANAGER, 1, 1),
            (CHEMISTRY_USER, 1, 1),
            (BIOLOGY_MANAGER, 1, 0),
            (BIOLOGY_USER, 1, 0),
            (NEW_USER, 0, 0),
            (ALL_READER, 2, 1),
            (PRODUCT_EDITOR, 0, 0),
        ] {
            let filter = RequestFilter {
                id: Some(ETHANOL),
                ..Default::default()
            };
            let product = get_products(&db, filter, person_id).unwrap().0.remove(0);
            assert_eq!(product.product_sc, Some(current), "{person_id}");
            assert_eq!(product.product_asc, Some(archived), "{person_id}");
            assert_eq!(product.product_tsc, Some(2), "{person_id}");
        }
    }

    #[test]
    fn test_get_products_barecodes_per_profile() {
        let db = crate::test_utils::init_test_with_fixture();

        // The barcodes of the Ethanol storages the person reads, without
        // their copy number.
        for (person_id, expected) in [
            (ADMIN, vec!["CA2", "CB2"]),
            (CHEMISTRY_USER, vec!["CA2"]),
            (BIOLOGY_USER, vec!["CB2"]),
            (ALL_READER, vec!["CA2", "CB2"]),
            (NEW_USER, vec![]),
        ] {
            let filter = RequestFilter {
                id: Some(ETHANOL),
                ..Default::default()
            };
            let product = get_products(&db, filter, person_id).unwrap().0.remove(0);
            let mut barecodes: Vec<String> = product
                .product_sl
                .unwrap_or_default()
                .split_whitespace()
                .map(String::from)
                .collect();
            barecodes.sort();
            assert_eq!(barecodes, expected, "{person_id}");
        }
    }

    #[test]
    fn test_get_products_availability_per_profile() {
        let db = crate::test_utils::init_test_with_fixture();

        // The entities, with a manager, holding Ethanol: shown to anyone who
        // reads storages, whatever their entity, to know whom to ask.
        for (person_id, expected) in [
            (ADMIN, vec!["Biology", "Chemistry"]),
            (CHEMISTRY_USER, vec!["Biology", "Chemistry"]),
            (BIOLOGY_USER, vec!["Biology", "Chemistry"]),
            (ALL_READER, vec!["Biology", "Chemistry"]),
            (NEW_USER, vec![]),
            (PRODUCT_EDITOR, vec![]),
        ] {
            let filter = RequestFilter {
                id: Some(ETHANOL),
                ..Default::default()
            };
            let product = get_products(&db, filter, person_id).unwrap().0.remove(0);
            let mut entities: Vec<String> = product
                .product_availability
                .unwrap_or_default()
                .into_iter()
                .map(|entity| entity.entity_name)
                .collect();
            entities.sort();
            assert_eq!(entities, expected, "{person_id}");
        }
    }

    #[test]
    fn test_export_products_permissions() {
        let db = crate::test_utils::init_test_with_fixture();

        // A header, and a row per product the person reads.
        for (person_id, rows) in [(ADMIN, 6), (CHEMISTRY_USER, 5), (PRODUCT_EDITOR, 6)] {
            let csv = export_products(&db, RequestFilter::default(), person_id).unwrap();
            assert_eq!(csv.lines().count(), rows + 1, "{person_id}");
        }
    }

    #[test]
    fn test_get_products_with_name_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        // By name, or by synonym: ETHYL ALCOHOL (7) is a synonym of Ethanol.
        for name_id in [2, 7] {
            let filter = RequestFilter {
                name: Some(name_id),
                ..Default::default()
            };
            assert_eq!(product_ids(&db, filter, 1), [ETHANOL], "{name_id}");
        }
    }

    #[test]
    fn test_get_products_with_type_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        for (filter, expected) in [
            (
                RequestFilter {
                    show_chem: true,
                    ..Default::default()
                },
                vec![1, 2, 3, 4],
            ),
            (
                RequestFilter {
                    show_bio: true,
                    ..Default::default()
                },
                vec![5],
            ),
            (
                RequestFilter {
                    show_consu: true,
                    ..Default::default()
                },
                vec![6],
            ),
        ] {
            assert_eq!(product_ids(&db, filter, 1), expected);
        }
    }

    #[test]
    fn test_get_products_with_field_filters() {
        let db = crate::test_utils::init_test_with_fixture();

        let filter = RequestFilter {
            category: Some(id_of(&db, "category", "category_label", "Antibody")),
            ..Default::default()
        };
        assert_eq!(product_ids(&db, filter, 1), [5]);

        let filter = RequestFilter {
            signal_word: Some(id_of(&db, "signal_word", "signal_word_label", "danger")),
            ..Default::default()
        };
        assert_eq!(product_ids(&db, filter, 1), [2, 4]);

        let filter = RequestFilter {
            producer_ref: Some(1),
            ..Default::default()
        };
        assert_eq!(product_ids(&db, filter, 1), [ETHANOL]);

        let filter = RequestFilter {
            cas_number: Some(id_of(&db, "cas_number", "cas_number_label", "7732-18-5")),
            ..Default::default()
        };
        assert_eq!(product_ids(&db, filter, 1), [1]);
    }

    #[test]
    fn test_get_products_with_link_filters() {
        let db = crate::test_utils::init_test_with_fixture();
        let h225 = id_of(
            &db,
            "hazard_statement",
            "hazard_statement_reference",
            "H225",
        );
        let h319 = id_of(
            &db,
            "hazard_statement",
            "hazard_statement_reference",
            "H319",
        );
        let p210 = id_of(
            &db,
            "precautionary_statement",
            "precautionary_statement_reference",
            "P210",
        );
        let ghs02 = id_of(&db, "symbol", "symbol_label", "GHS02");
        let ghs07 = id_of(&db, "symbol", "symbol_label", "GHS07");
        let solvent = id_of(&db, "tag", "tag_label", "Solvent");

        for (filter, expected) in [
            (
                RequestFilter {
                    hazard_statements: Some(vec![h225]),
                    ..Default::default()
                },
                vec![2, 4],
            ),
            (
                RequestFilter {
                    hazard_statements: Some(vec![h319]),
                    ..Default::default()
                },
                vec![2],
            ),
            (
                RequestFilter {
                    precautionary_statements: Some(vec![p210]),
                    ..Default::default()
                },
                vec![2],
            ),
            (
                RequestFilter {
                    symbols: Some(vec![ghs02]),
                    ..Default::default()
                },
                vec![2, 4],
            ),
            (
                RequestFilter {
                    symbols: Some(vec![ghs07]),
                    ..Default::default()
                },
                vec![2],
            ),
            (
                RequestFilter {
                    tags: Some(vec![solvent]),
                    ..Default::default()
                },
                vec![2],
            ),
        ] {
            assert_eq!(product_ids(&db, filter.clone(), 1), expected, "{filter:?}");
        }
    }

    #[test]
    fn test_get_products_with_bookmark_filter() {
        let db = crate::test_utils::init_test_with_fixture();
        let filter = RequestFilter {
            bookmark: true,
            ..Default::default()
        };

        // chemistry.user bookmarked Ethanol, the admin nothing.
        assert_eq!(product_ids(&db, filter.clone(), 3), [ETHANOL]);
        assert_eq!(product_ids(&db, filter, 1), [0u64; 0]);
    }

    #[test]
    fn test_get_products_with_product_specificity_filter() {
        let db = crate::test_utils::init_test_with_fixture();

        // LIKE is case-insensitive for ASCII.
        for (specificity, expected) in [("ABS", vec![2]), ("distilled", vec![1]), ("none", vec![])]
        {
            let filter = RequestFilter {
                product_specificity: Some(specificity.to_string()),
                ..Default::default()
            };
            assert_eq!(product_ids(&db, filter, 1), expected, "{specificity}");
        }
    }

    #[test]
    fn test_get_products_loads_the_product() {
        let db = crate::test_utils::init_test_with_fixture();

        let product = get_product(&db, ETHANOL);

        assert_eq!(product.name.name_label, "ETHANOL");
        assert_eq!(product.product_type, ProductType::Chem);
        assert_eq!(product.cas_number.unwrap().cas_number_label, "64-17-5");
        assert_eq!(product.ce_number.unwrap().ce_number_label, "200-578-6");
        assert_eq!(
            product.linear_formula.unwrap().linear_formula_label,
            "CH3CH2OH"
        );
        assert_eq!(
            product.physical_state.unwrap().physical_state_label,
            "liquid"
        );
        assert_eq!(product.product_molecular_weight, Some(46.07));
        assert_eq!(product.unit_molecular_weight.unwrap().unit_label, "g/mol");
        assert_eq!(product.unit_temperature.unwrap().unit_label, "°C");
        assert_eq!(
            product.unit_concentration.unwrap().unit_type,
            UnitType::Concentration
        );
        assert_eq!(
            product.producer_ref.unwrap().producer.producer_label,
            "Acros Organics"
        );
        assert_eq!(product.person.person_email, "chemistry.manager@example.com");
        // Storages: 2 in Chemistry and 7 in Biology; 3 (Chemistry) is archived
        // and the history row 4 does not count.
        assert_eq!(product.product_tsc, Some(2));
        assert_eq!(product.product_asc, Some(1));
    }

    #[test]
    fn test_create_product() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let acros = id_of(&db, "supplier", "supplier_label", "Acros Organics");
        let product = ProductStruct {
            name: NameStruct {
                name_label: "ISOPROPANOL".to_string(),
                ..Default::default()
            },
            person: PersonStruct {
                person_id: Some(2),
                ..Default::default()
            },
            product_type: ProductType::Chem,
            product_inchi: Some("InChI=1S/C3H8O/c1-3(2)4/h3-4H,1-2H3".to_string()),
            product_inchikey: Some("KFZMGEQAYNKOFK-UHFFFAOYSA-N".to_string()),
            product_canonical_smiles: Some("CC(C)O".to_string()),
            product_specificity: Some("anhydrous".to_string()),
            product_molecular_weight: Some(60.1),
            product_temperature: Some(25.0),
            product_number_per_carton: Some(6),
            cas_number: Some(CasNumberStruct {
                cas_number_label: "67-63-0".to_string(),
                ..Default::default()
            }),
            empirical_formula: Some(EmpiricalFormulaStruct {
                empirical_formula_label: "C3H8O".to_string(),
                ..Default::default()
            }),
            signal_word: Some(SignalWordStruct {
                signal_word_id: Some(id_of(&db, "signal_word", "signal_word_label", "danger")),
                ..Default::default()
            }),
            category: Some(CategoryStruct {
                category_id: Some(id_of(&db, "category", "category_label", "Drug")),
                ..Default::default()
            }),
            producer_ref: Some(ProducerRefStruct {
                producer_ref_id: Some(3),
                ..Default::default()
            }),
            synonyms: Some(vec![NameStruct {
                name_label: "ISOPROPYL ALCOHOL".to_string(),
                ..Default::default()
            }]),
            classes_of_compound: Some(vec![ClassOfCompoundStruct {
                class_of_compound_label: "alcohol".to_string(),
                ..Default::default()
            }]),
            symbols: Some(vec![SymbolStruct {
                symbol_id: Some(id_of(&db, "symbol", "symbol_label", "GHS02")),
                ..Default::default()
            }]),
            tags: Some(vec![TagStruct {
                tag_label: "Solvent".to_string(),
                ..Default::default()
            }]),
            hazard_statements: Some(vec![HazardStatementStruct {
                hazard_statement_id: Some(id_of(
                    &db,
                    "hazard_statement",
                    "hazard_statement_reference",
                    "H225",
                )),
                ..Default::default()
            }]),
            precautionary_statements: Some(vec![PrecautionaryStatementStruct {
                precautionary_statement_id: Some(id_of(
                    &db,
                    "precautionary_statement",
                    "precautionary_statement_reference",
                    "P210",
                )),
                ..Default::default()
            }]),
            supplier_refs: Some(vec![SupplierRefStruct {
                supplier_ref_label: "A-4242".to_string(),
                supplier: SupplierStruct {
                    supplier_id: Some(acros),
                    ..Default::default()
                },
                ..Default::default()
            }]),
            unit_temperature: Some(UnitStruct {
                unit_id: Some(id_of(&db, "unit", "unit_label", "°C")),
                ..Default::default()
            }),
            unit_molecular_weight: Some(UnitStruct {
                unit_id: Some(id_of(&db, "unit", "unit_label", "g/mol")),
                ..Default::default()
            }),
            ..Default::default()
        };

        let product_id = create_update_product(&mut db, product).unwrap();
        assert_eq!(product_id, 7);

        let product = get_product(&db, 7);
        assert_eq!(product.name.name_label, "ISOPROPANOL");
        assert_eq!(product.cas_number.unwrap().cas_number_label, "67-63-0");
        assert_eq!(
            product.empirical_formula.unwrap().empirical_formula_label,
            "C3H8O"
        );
        assert_eq!(product.category.unwrap().category_label, "Drug");
        assert_eq!(
            product.producer_ref.unwrap().producer_ref_label,
            "ACS reagent"
        );
        assert_eq!(product.unit_temperature.unwrap().unit_label, "°C");
        assert_eq!(
            sorted_labels(&product.synonyms.unwrap(), |item| &item.name_label),
            ["ISOPROPYL ALCOHOL"]
        );
        assert_eq!(
            sorted_labels(&product.symbols.unwrap(), |item| &item.symbol_label),
            ["GHS02"]
        );
        assert_eq!(
            sorted_labels(&product.tags.unwrap(), |item| &item.tag_label),
            ["Solvent"]
        );
        assert_eq!(
            sorted_labels(&product.supplier_refs.unwrap(), |item| &item
                .supplier_ref_label),
            ["A-4242"]
        );
        // The existing class of compound and tag are reused.
        let count = |sql: &str| -> i64 { db.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM class_of_compound WHERE class_of_compound_label = 'alcohol'"
            ),
            1
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM tag WHERE tag_label = 'Solvent'"),
            1
        );
    }

    #[test]
    fn test_update_product() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Water (1) renamed, with a new name.
        let mut product = get_product(&db, 1);
        product.name = NameStruct {
            name_label: "DEMINERALIZED WATER".to_string(),
            ..Default::default()
        };
        product.product_specificity = None;
        assert_eq!(create_update_product(&mut db, product).unwrap(), 1);

        let product = get_product(&db, 1);
        assert_eq!(product.name.name_label, "DEMINERALIZED WATER");
        assert_eq!(product.product_specificity, None);
        assert_eq!(product.cas_number.unwrap().cas_number_label, "7732-18-5");
    }

    #[test]
    fn test_delete_product() {
        let mut db = crate::test_utils::init_test_with_fixture();

        // Ethanol has storages: the database refuses, the handler checks it
        // first with `product_has_storages`.
        assert!(product_has_storages(&db, ETHANOL).unwrap());
        assert!(delete_product(&mut db, ETHANOL).is_err());

        // Sodium chloride (3) has none.
        assert!(!product_has_storages(&db, 3).unwrap());
        delete_product(&mut db, 3).unwrap();
        let filter = RequestFilter {
            id: Some(3),
            ..Default::default()
        };
        assert_eq!(product_ids(&db, filter, 1), [0u64; 0]);
    }

    #[test]
    fn test_export_products() {
        let db = crate::test_utils::init_test_with_fixture();

        // All the products the admin reads.
        let csv = export_products(&db, RequestFilter::default(), 1).unwrap();
        assert_eq!(csv.lines().count(), 7);

        // Every field of Ethanol, by column.
        let filter = RequestFilter {
            id: Some(ETHANOL),
            ..Default::default()
        };
        let csv = export_products(&db, filter, 1).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(headers.len(), 40);
        assert_eq!(records.len(), 1);
        let value = |column: &str| {
            let index = headers.iter().position(|header| header == column).unwrap();
            records[0][index].to_string()
        };
        let sorted_list = |column: &str| {
            let mut items: Vec<String> = value(column).split('|').map(String::from).collect();
            items.sort();
            items
        };

        for (column, expected) in [
            ("ID", "2"),
            ("NAME", "ETHANOL"),
            ("PRODUCT_SPECIFICITY", "absolute"),
            ("PRODUCT_TYPE", "chem"),
            ("CAS_NUMBER", "64-17-5"),
            ("CE_NUMBER", "200-578-6"),
            ("EMPIRICAL_FORMULA", "C2H6O"),
            ("SYNONYMS", "ETHYL ALCOHOL"),
            ("LINEAR_FORMULA", "CH3CH2OH"),
            ("PHYSICAL_STATE", "liquid"),
            ("SIGNAL_WORD", "danger"),
            ("PRECAUTIONARY_STATEMENTS", "P210"),
            ("CLASSES_OF_COMPOUND", "alcohol"),
            ("PRODUCER", "Acros Organics"),
            ("PRODUCER_REF", "ETOH-96"),
            ("SUPPLIER_REFS", "Acros Organics:A-1234"),
            ("TAGS", "Solvent"),
            ("MOLECULAR_WEIGHT", "46.07"),
            ("MOLECULAR_WEIGHT_UNIT", "g/mol"),
            ("TEMPERATURE", "20"),
            ("TEMPERATURE_UNIT", "°C"),
            ("CONCENTRATION", "96"),
            ("CONCENTRATION_UNIT", "%"),
            ("RESTRICTED", "false"),
            ("PERSON", "chemistry.manager@example.com"),
        ] {
            assert_eq!(value(column), expected, "{column}");
        }
        assert_eq!(sorted_list("SYMBOLS"), ["GHS02", "GHS07"]);
        assert_eq!(sorted_list("HAZARD_STATEMENTS"), ["H225", "H319"]);
    }

    // Ethanol, to populate.
    fn ethanol() -> Vec<ProductStruct> {
        vec![ProductStruct {
            product_id: Some(ETHANOL),
            ..Default::default()
        }]
    }

    #[test]
    fn test_populate_links() {
        let db = crate::test_utils::init_test_with_fixture();

        let mut products = ethanol();
        populate_synonyms(&db, &mut products).unwrap();
        populate_classes_of_compound(&db, &mut products).unwrap();
        populate_symbols(&db, &mut products).unwrap();
        populate_hazard_statements(&db, &mut products).unwrap();
        populate_precautionary_statements(&db, &mut products).unwrap();
        populate_supplier_refs(&db, &mut products).unwrap();
        populate_tags(&db, &mut products).unwrap();
        let product = products.remove(0);

        assert_eq!(
            sorted_labels(&product.synonyms.unwrap(), |item| &item.name_label),
            ["ETHYL ALCOHOL"]
        );
        assert_eq!(
            sorted_labels(&product.classes_of_compound.unwrap(), |item| &item
                .class_of_compound_label),
            ["alcohol"]
        );
        assert_eq!(
            sorted_labels(&product.symbols.unwrap(), |item| &item.symbol_label),
            ["GHS02", "GHS07"]
        );
        assert_eq!(
            sorted_labels(&product.hazard_statements.unwrap(), |item| &item
                .hazard_statement_reference),
            ["H225", "H319"]
        );
        assert_eq!(
            sorted_labels(&product.precautionary_statements.unwrap(), |item| {
                &item.precautionary_statement_reference
            }),
            ["P210"]
        );
        assert_eq!(
            sorted_labels(&product.supplier_refs.unwrap(), |item| &item
                .supplier_ref_label),
            ["A-1234"]
        );
        assert_eq!(
            sorted_labels(&product.tags.unwrap(), |item| &item.tag_label),
            ["Solvent"]
        );
    }

    #[test]
    fn test_populate_links_of_a_product_without_links() {
        let db = crate::test_utils::init_test_with_fixture();

        // Water (1) has none, product 999 does not exist.
        for product_id in [1, 999] {
            let mut products = vec![ProductStruct {
                product_id: Some(product_id),
                ..Default::default()
            }];
            populate_symbols(&db, &mut products).unwrap();
            populate_tags(&db, &mut products).unwrap();
            assert!(products[0].symbols.is_none(), "{product_id}");
            assert!(products[0].tags.is_none(), "{product_id}");
        }
    }

    #[test]
    fn test_product_has_storages() {
        let db = crate::test_utils::init_test_with_fixture();

        assert!(product_has_storages(&db, ETHANOL).unwrap());
        assert!(product_has_storages(&db, 4).unwrap());
        assert!(!product_has_storages(&db, 3).unwrap());
        assert!(!product_has_storages(&db, 6).unwrap());
    }

    #[test]
    fn test_update_product_returns_link_errors() {
        // A failing removal of the old links, for each link table.
        for link_table in [
            "productsymbols",
            "producttags",
            "productsynonyms",
            "productsupplierrefs",
            "producthazardstatements",
            "productprecautionarystatements",
            "productclassesofcompounds",
        ] {
            let mut db = crate::test_utils::init_test_with_fixture();
            db.execute_batch(&format!(
                "CREATE TRIGGER fail_delete BEFORE DELETE ON {link_table}
                 BEGIN SELECT RAISE(ABORT, 'delete failed'); END;"
            ))
            .unwrap();

            // Ethanol saved without links: only the removal of the old links
            // runs.
            let product = ProductStruct {
                product_id: Some(ETHANOL),
                name: NameStruct {
                    name_id: Some(2),
                    name_label: "ETHANOL".to_string(),
                    ..Default::default()
                },
                person: PersonStruct {
                    person_id: Some(1),
                    ..Default::default()
                },
                ..Default::default()
            };

            let error = create_update_product(&mut db, product).unwrap_err();
            assert!(
                error.to_string().contains("delete failed"),
                "{link_table}: {error}"
            );
        }
    }

    #[test]
    fn test_create_update_product_concentration() {
        let mut db = crate::test_utils::init_test_with_fixture();
        let mg_per_l = id_of(&db, "unit", "unit_label", "mg/L");

        // Water (1): 1.5 mg/L.
        let mut product = get_product(&db, 1);
        product.product_concentration = Some(1.5);
        product.unit_concentration = Some(UnitStruct {
            unit_id: Some(mg_per_l),
            ..Default::default()
        });
        create_update_product(&mut db, product).unwrap();

        let product = get_product(&db, 1);
        assert_eq!(product.product_concentration, Some(1.5));
        let unit = product.unit_concentration.clone().unwrap();
        assert_eq!(unit.unit_label, "mg/L");
        assert_eq!(unit.unit_type, UnitType::Concentration);

        // Cleared.
        let mut product = product;
        product.product_concentration = None;
        product.unit_concentration = None;
        create_update_product(&mut db, product).unwrap();

        let product = get_product(&db, 1);
        assert_eq!(product.product_concentration, None);
        assert!(product.unit_concentration.is_none());
    }
}
