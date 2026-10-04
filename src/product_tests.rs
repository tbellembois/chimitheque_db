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
    use rusqlite::{Batch, Connection, fallible_iterator::FallibleIterator};
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

        let db = crate::test_utils::init_test();
        let sql = include_str!("resources/test.sql");
        let mut batch = Batch::new(&db, sql);
        while let Ok(Some(mut stmt)) = batch.next() {
            stmt.execute([]).unwrap();
        }

        db
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
            name: Some(1),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 3);
        assert_eq!(products.len(), 3);
        assert_eq!(products[0].name.name_label, "PRODUCT_NAME-0001");
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
            category: Some(4),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 3);
        assert_eq!(products.len(), 3);
        assert_eq!(
            products[0].clone().category.unwrap().category_label,
            "Cellular Growth Factor"
        );
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
        assert_eq!(count, 11);
        assert_eq!(products.len(), 11);
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
        assert_eq!(count, 8);
        assert_eq!(products.len(), 8);
    }

    #[test]
    fn test_get_products_with_tag_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            tags: Some(vec![1]),
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
        assert_eq!(count, 120);
        assert_eq!(products.len(), 120);
    }

    #[test]
    fn test_get_products_with_producer_ref_filter() {
        let db = init_test_products();
        let filter = chimitheque_types::requestfilter::RequestFilter {
            producer_ref: Some(1),
            ..Default::default()
        };
        let person_id = 1;

        let result = get_products(&db, filter, person_id);
        assert!(result.is_ok());

        let (products, count) = result.unwrap();
        assert_eq!(count, 3);
        assert_eq!(products.len(), 3);
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
    fn test_create_product2() {
        let mut db = init_test_products();
        let product = ProductStruct {
            product_id: None,
            name: NameStruct {
                name_id: None,
                name_label: "This is a new product".to_string(),
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
                name_label: "This is a new product".to_string(),
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

        let result = delete_product(&mut db, 1);
        assert!(result.is_ok());

        // Verify the product was deleted
        let filter = chimitheque_types::requestfilter::RequestFilter {
            id: Some(1),
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

        assert_eq!(products[0].synonyms.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].synonyms.as_ref().unwrap()[0].name_label,
            "PRODUCT_NAME-0426"
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

        assert_eq!(products[0].classes_of_compound.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].classes_of_compound.as_ref().unwrap()[0].class_of_compound_label,
            "CLASS_OF-0220"
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

        assert_eq!(products[0].symbols.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].symbols.as_ref().unwrap()[0].symbol_label,
            "GHS02"
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

        assert_eq!(products[0].hazard_statements.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].hazard_statements.as_ref().unwrap()[0].hazard_statement_reference,
            "H221"
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
            products[0].precautionary_statements.as_ref().unwrap().len(),
            1
        );
        assert_eq!(
            products[0].precautionary_statements.as_ref().unwrap()[0]
                .precautionary_statement_reference,
            "P230"
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

        assert_eq!(products[0].supplier_refs.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].supplier_refs.as_ref().unwrap()[0].supplier_ref_label,
            "3Bkl eXLQpTgsoTSQwSpyo PuBZnEUdq-JlVlfLzwRkxr"
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

        assert_eq!(products[0].tags.as_ref().unwrap().len(), 1);
        assert_eq!(
            products[0].tags.as_ref().unwrap()[0].tag_label,
            "TAG_LABE-0372"
        );
    }
}
