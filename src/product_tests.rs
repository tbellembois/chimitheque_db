#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::too_many_lines
    )]

    use crate::product::*;
    use chimitheque_types::{
        casnumber::CasNumber as CasNumberStruct, category::Category as CategoryStruct,
        classofcompound::ClassOfCompound as ClassOfCompoundStruct,
        empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct,
        hazardstatement::HazardStatement as HazardStatementStruct, name::Name as NameStruct,
        person::Person as PersonStruct,
        precautionarystatement::PrecautionaryStatement as PrecautionaryStatementStruct,
        product::Product as ProductStruct, producttype::ProductType,
        signalword::SignalWord as SignalWordStruct, supplier::Supplier as SupplierStruct,
        supplierref::SupplierRef as SupplierRefStruct, symbol::Symbol as SymbolStruct,
        tag::Tag as TagStruct, unittype::UnitType,
    };
    use rusqlite::Connection;
    use std::io::Write;

    fn init_test_products() -> Connection {
        let _ = env_logger::Builder::from_default_env()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{}:{}] {} - {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.level(),
                    record.args()
                )
            })
            .try_init();

        crate::test_utils::init_test_with_data()
    }

    /// Labels of the populated items, sorted: the populate queries have no
    /// ORDER BY, so the item order is not guaranteed.
    fn sorted_labels<T>(items: &[T], label: impl Fn(&T) -> &String) -> Vec<String> {
        let mut labels: Vec<String> = items.iter().map(|item| label(item).clone()).collect();
        labels.sort();
        labels
    }

    #[test]
    fn test_get_products() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 1000);
        assert_eq!(products.len(), 1000);
    }

    #[test]
    fn test_get_products_with_name_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            name: Some(408),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        // 5 products named PRODUCT_NAME-00408, and 2 having it as synonym.
        assert_eq!(count, 7);
        assert_eq!(products.len(), 7);
        assert_eq!(
            products
                .iter()
                .filter(|product| product.name.name_label == "PRODUCT_NAME-00408")
                .count(),
            5
        );
    }

    #[test]
    fn test_get_products_with_type_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            show_chem: true,
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 327);
        assert_eq!(products.len(), 327);
        assert_eq!(products[0].product_type, ProductType::Chem);
    }

    #[test]
    fn test_get_products_with_category_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            category: Some(3),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 1);
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].clone().category.unwrap().category_label, "Drug");
    }

    #[test]
    fn test_get_products_with_signal_word_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            signal_word: Some(1),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 489);
        assert_eq!(products.len(), 489);
        assert_eq!(
            products[0].clone().signal_word.unwrap().signal_word_label,
            "danger"
        );
    }

    #[test]
    fn test_get_products_with_hazard_statement_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            hazard_statements: Some(vec![1]),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 23);
        assert_eq!(products.len(), 23);
    }

    #[test]
    fn test_get_products_with_precautionary_statement_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            precautionary_statements: Some(vec![1]),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 17);
        assert_eq!(products.len(), 17);
    }

    #[test]
    fn test_get_products_with_tag_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            tags: Some(vec![6]),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 2);
        assert_eq!(products.len(), 2);
    }

    #[test]
    fn test_get_products_with_symbol_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            symbols: Some(vec![1]),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 223);
        assert_eq!(products.len(), 223);
    }

    #[test]
    fn test_get_products_with_producer_ref_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            producer_ref: Some(46),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 5);
        assert_eq!(products.len(), 5);
    }

    #[test]
    fn test_get_products_with_bookmark_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            bookmark: true,
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 1);
        assert_eq!(products.len(), 1);
    }

    #[test]
    fn test_create_product() {
        let mut db = init_test_products();
        let product = ProductStruct {
            product_id: None,
            name: NameStruct {
                name_id: None,
                name_label: "THIS IS A NEW PRODUCT".to_string(),
                ..Default::default()
            },
            person: PersonStruct {
                person_id: Some(1),
                ..Default::default()
            },
            product_type: ProductType::Chem,
            product_inchi: Some("InChI=1S/C2H6O/c1-2-3/h3H,2H2,1H3".to_string()),
            product_inchikey: Some("LFQSCWFLJHTTHZ-UHFFFAOYSA-N".to_string()),
            product_canonical_smiles: Some("CCO".to_string()),
            product_specificity: Some("High purity".to_string()),
            product_msds: Some("MSDS003".to_string()),
            product_restricted: false,
            product_radioactive: false,
            product_twod_formula: Some("C2H5OH".to_string()),
            product_threed_formula: Some("3D_ETH".to_string()),
            product_disposal_comment: Some("Dispose properly".to_string()),
            product_remark: Some("High quality".to_string()),
            product_molecular_weight: Some(46.07),
            product_temperature: Some(22.0),
            product_sheet: Some("Sheet003".to_string()),
            product_number_per_carton: Some(15),
            product_number_per_bag: Some(8),
            cas_number: Some(CasNumberStruct {
                cas_number_id: Some(3),
                cas_number_label: "64-17-5".to_string(),
                ..Default::default()
            }),
            ce_number: None,
            empirical_formula: Some(EmpiricalFormulaStruct {
                empirical_formula_id: Some(3),
                empirical_formula_label: "C2H6O".to_string(),
                ..Default::default()
            }),
            linear_formula: None,
            physical_state: None,
            signal_word: Some(SignalWordStruct {
                signal_word_id: Some(1),
                signal_word_label: "Danger".to_string(),
                ..Default::default()
            }),
            category: Some(CategoryStruct {
                category_id: Some(1),
                category_label: "Chemical".to_string(),
                ..Default::default()
            }),
            producer_ref: Some(ProducerRefStruct {
                producer_ref_id: Some(1),
                producer_ref_label: "PROD001".to_string(),
                ..Default::default()
            }),
            synonyms: Some(vec![NameStruct {
                name_id: None,
                name_label: "Ethyl alcohol".to_string(),
                ..Default::default()
            }]),
            classes_of_compound: Some(vec![ClassOfCompoundStruct {
                class_of_compound_id: None,
                class_of_compound_label: "Alcohol".to_string(),
                ..Default::default()
            }]),
            symbols: Some(vec![SymbolStruct {
                symbol_id: Some(2),
                symbol_label: "GHS02".to_string(),
                ..Default::default()
            }]),
            tags: Some(vec![TagStruct {
                tag_id: None,
                tag_label: "Organic".to_string(),
                ..Default::default()
            }]),
            hazard_statements: Some(vec![HazardStatementStruct {
                hazard_statement_id: Some(18),
                hazard_statement_label: "Highly Flammable liquid and vapor".to_string(),
                hazard_statement_reference: "H225".to_string(),
                ..Default::default()
            }]),
            precautionary_statements: Some(vec![PrecautionaryStatementStruct {
                precautionary_statement_id: Some(7),
                precautionary_statement_label: "Keep away from heat, hot surface, sparks, open flames and other ignition sources. No smoking.".to_string(),
                precautionary_statement_reference: "P210".to_string(),
                ..Default::default()
            }]),
            supplier_refs: Some(vec![SupplierRefStruct {
                supplier_ref_id: None,
                supplier_ref_label: "REF001".to_string(),
                supplier: SupplierStruct {
                    supplier_id: Some(1),
                    supplier_label: "Abcam".to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            }]),
            unit_temperature: Some(UnitStruct {
                unit_id: Some(1),
                unit_label: "Celsius".to_string(),
                unit_multiplier: 1.0,
                unit_type: UnitType::Temperature,
                unit: None,
            }),
            unit_molecular_weight: Some(UnitStruct {
                unit_id: Some(3),
                unit_label: "g/mol".to_string(),
                unit_multiplier: 1.0,
                unit_type: UnitType::MolecularWeight,
                unit: None,
            }),
            ..Default::default()
        };

        let result = create_update_product(&mut db, product);
        assert!(result.is_ok());

        let product_id = result.unwrap();
        assert!(product_id > 0);

        // Verify the product was created
        let filter = chimitheque_types::requestfilter::RequestFilter {
            id: Some(product_id),
            ..Default::default()
        };
        let (products, _) = get_products(&db, filter, 1).unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].name.name_label, "THIS IS A NEW PRODUCT");
    }

    #[test]
    fn test_update_product() {
        let mut db = init_test_products();
        let product = ProductStruct {
            product_id: Some(1),
            name: NameStruct {
                name_id: None,
                name_label: "THIS IS A NEW PRODUCT".to_string(),
                ..Default::default()
            },
            person: PersonStruct {
                person_id: Some(1),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = create_update_product(&mut db, product);
        assert!(result.is_ok());

        let product_id = result.unwrap();
        assert_eq!(product_id, 1);

        // Verify the product was updated
        let filter = chimitheque_types::requestfilter::RequestFilter {
            id: Some(1),
            ..Default::default()
        };
        let (products, _) = get_products(&db, filter, 1).unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].name.name_label, "THIS IS A NEW PRODUCT");
    }

    #[test]
    fn test_delete_product() {
        let mut db = init_test_products();

        // Product 1 has storages: the database refuses, the handler checks it
        // first with `product_has_storages`.
        assert!(product_has_storages(&db, 1).unwrap());
        assert!(delete_product(&mut db, 1).is_err());

        // Product 2 has none.
        assert!(!product_has_storages(&db, 2).unwrap());
        let result = delete_product(&mut db, 2);
        assert!(result.is_ok());

        // Verify the product was deleted
        let filter = chimitheque_types::requestfilter::RequestFilter {
            id: Some(2),
            ..Default::default()
        };
        let result = get_products(&db, filter, 1);
        assert!(result.is_ok());

        let (products, _) = result.unwrap();
        assert_eq!(products.len(), 0);
    }

    #[test]
    fn test_export_products() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            ..Default::default()
        };
        let person_id = 1;

        let result = export_products(&db, filter, person_id);
        debug!("{result:?}");
        assert!(result.is_ok());

        // Every field of product 1, by column.
        let filter = chimitheque_types::requestfilter::RequestFilter {
            id: Some(1),
            ..Default::default()
        };
        let csv = export_products(&db, filter, person_id).unwrap();
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

        assert_eq!(value("ID"), "1");
        assert_eq!(value("NAME"), "PRODUCT_NAME-00859");
        assert_eq!(value("PRODUCT_TYPE"), "chem");
        assert_eq!(value("CATEGORY"), "CATEGORY-00540");
        assert_eq!(sorted_list("SYMBOLS"), ["GHS01", "GHS02"]);
        assert_eq!(sorted_list("HAZARD_STATEMENTS"), ["H200", "H201"]);
        assert_eq!(sorted_list("TAGS"), ["3D Cell Culture", "Acid"]);
        assert_eq!(
            sorted_list("SYNONYMS"),
            ["PRODUCT_NAME-00001", "PRODUCT_NAME-00002"]
        );
        assert_eq!(value("MOLECULAR_WEIGHT"), "879234.4");
        assert_eq!(value("MOLECULAR_WEIGHT_UNIT"), "L");
        assert_eq!(value("TEMPERATURE"), "279501.13");
        assert_eq!(value("TEMPERATURE_UNIT"), "mg");
        assert_eq!(value("CONCENTRATION"), "806309.52");
        assert_eq!(value("CONCENTRATION_UNIT"), "nM");
        assert_eq!(value("RESTRICTED"), "true");
        assert_eq!(value("PERSON"), "wallacemichael@example.com");
    }

    #[test]
    fn test_populate_synonyms() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_synonyms(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].synonyms.as_ref().unwrap(), |item| &item
                .name_label),
            ["PRODUCT_NAME-00001", "PRODUCT_NAME-00002"]
        );
    }

    #[test]
    fn test_populate_classes_of_compound() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_classes_of_compound(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].classes_of_compound.as_ref().unwrap(), |item| {
                &item.class_of_compound_label
            }),
            ["acetal", "acetophenone"]
        );
    }

    #[test]
    fn test_populate_symbols_non_existing() {
        let db = init_test_products();
        let product = ProductStruct {
            product_id: Some(1001),
            ..Default::default()
        };

        let result = populate_symbols(&db, &mut [product]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_populate_symbols_existing() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_symbols(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].symbols.as_ref().unwrap(), |item| &item
                .symbol_label),
            ["GHS01", "GHS02"]
        );
    }

    #[test]
    fn test_populate_hazard_statements() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_hazard_statements(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].hazard_statements.as_ref().unwrap(), |item| {
                &item.hazard_statement_reference
            }),
            ["H200", "H201"]
        );
    }

    #[test]
    fn test_populate_precautionary_statements() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_precautionary_statements(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(
                products[0].precautionary_statements.as_ref().unwrap(),
                |item| &item.precautionary_statement_reference
            ),
            ["P101", "P102"]
        );
    }

    #[test]
    fn test_populate_supplier_refs() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_supplier_refs(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].supplier_refs.as_ref().unwrap(), |item| &item
                .supplier_ref_label),
            ["Il0IneWQ", "gurRQFua3lWiaO5nflsqcL39H8"]
        );
    }

    #[test]
    fn test_populate_tags() {
        let db = init_test_products();
        let mut products = vec![ProductStruct {
            product_id: Some(1),
            ..Default::default()
        }];

        let result = populate_tags(&db, &mut products);
        assert!(result.is_ok());

        assert_eq!(
            sorted_labels(products[0].tags.as_ref().unwrap(), |item| &item.tag_label),
            ["3D Cell Culture", "Acid"]
        );
    }

    #[test]
    fn test_get_products_with_product_specificity_filter() {
        let db = init_test_products();

        // Specificity contains "ab" (LIKE is case-insensitive for ASCII).
        let filter = chimitheque_types::requestfilter::RequestFilter {
            product_specificity: Some("ab".to_string()),
            ..Default::default()
        };
        let (products, count) = get_products(&db, filter, 1).unwrap();
        assert_eq!(count, 23);
        assert!(products.iter().all(|product| {
            product
                .product_specificity
                .as_deref()
                .is_some_and(|specificity| specificity.to_lowercase().contains("ab"))
        }));

        // The full specificity of product 1.
        let filter = chimitheque_types::requestfilter::RequestFilter {
            product_specificity: Some("Sdu1isVEkhXNsLXVqRpvX3hr6cS55M2".to_string()),
            ..Default::default()
        };
        let (products, count) = get_products(&db, filter, 1).unwrap();
        assert_eq!(count, 1);
        assert_eq!(products[0].product_id, Some(1));
    }

    #[test]
    fn test_product_has_storages() {
        let db_connection = init_test_products();
        let product_id =
            |sql: &str| -> u64 { db_connection.query_row(sql, [], |row| row.get(0)).unwrap() };
        let with_storages = product_id("SELECT product FROM storage LIMIT 1");
        let without_storages = product_id(
            "SELECT product_id FROM product WHERE product_id NOT IN (SELECT product FROM storage) LIMIT 1",
        );

        assert!(product_has_storages(&db_connection, with_storages).unwrap());
        assert!(!product_has_storages(&db_connection, without_storages).unwrap());
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
            let mut db = init_test_products();
            db.execute_batch(&format!(
                "CREATE TRIGGER fail_delete BEFORE DELETE ON {link_table}
                 BEGIN SELECT RAISE(ABORT, 'delete failed'); END;"
            ))
            .unwrap();

            // Without links: only the removal of the old links runs.
            let product = ProductStruct {
                product_id: Some(1),
                name: NameStruct {
                    name_label: "THIS IS A NEW PRODUCT".to_string(),
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
        let mut db = init_test_products();
        let get_product = |db: &Connection| {
            let filter = chimitheque_types::requestfilter::RequestFilter {
                id: Some(1),
                ..Default::default()
            };
            get_products(db, filter, 1).unwrap().0.remove(0)
        };

        // 1.5 mg/L.
        let mut product = get_product(&db);
        product.product_concentration = Some(1.5);
        product.unit_concentration = Some(UnitStruct {
            unit_id: Some(21),
            ..Default::default()
        });
        create_update_product(&mut db, product).unwrap();

        let product = get_product(&db);
        assert_eq!(product.product_concentration, Some(1.5));
        let unit = product.unit_concentration.clone().unwrap();
        assert_eq!(unit.unit_id, Some(21));
        assert_eq!(unit.unit_label, "mg/L");
        assert_eq!(unit.unit_type, UnitType::Concentration);

        // Cleared.
        let mut product = product;
        product.product_concentration = None;
        product.unit_concentration = None;
        create_update_product(&mut db, product).unwrap();

        let product = get_product(&db);
        assert_eq!(product.product_concentration, None);
        assert!(product.unit_concentration.is_none());
    }
}
