#[cfg(test)]
mod tests {

    use std::vec;

    use crate::{init::populate_db_with_base_data, pubchemproduct::*};
    use log::info;

    fn init_test_pubchemproduct() -> Connection {
        let mut db = crate::test_utils::init_test();
        populate_db_with_base_data(&mut db).unwrap();

        db
    }

    #[test]
    fn test_create_product_from_pubchem() {
        let mut db_connection = init_test_pubchemproduct();

        info!("testing create product from pubchem");
        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    ..Default::default()
                },
                1,
                None,
            )
            .is_ok_and(|id| id > 0)
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    ..Default::default()
                },
                1,
                None,
            )
            .is_err_and(|e| e.to_string().eq("empty name"))
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    hs: Some(vec!["foo".to_string()]),
                    ..Default::default()
                },
                1,
                None,
            )
            .is_err_and(|e| e.to_string().eq("unknown hazard statement foo"))
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    ps: Some(vec!["foo".to_string()]),
                    ..Default::default()
                },
                1,
                None,
            )
            .is_err_and(|e| e.to_string().eq("unknown precautionary statement foo"))
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    signal: Some(vec!["foo".to_string()]),
                    ..Default::default()
                },
                1,
                None,
            )
            .is_err_and(|e| e.to_string().eq("unknown signal word foo"))
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    molecular_weight_unit: Some("foo".to_string()),
                    ..Default::default()
                },
                1,
                None,
            )
            .is_err_and(|e| e.to_string().eq("unknown molecular weight unit foo"))
        );

        assert!(
            create_update_product_from_pubchem(
                &mut db_connection,
                PubchemProduct {
                    name: Some("aspirin".to_string()),
                    iupac_name: Some("iupac_name".to_string()),
                    inchi: Some("inchi".to_string()),
                    inchi_key: Some("inchi_key".to_string()),
                    canonical_smiles: Some("canonical_smiles".to_string()),
                    molecular_formula: Some("molecular_formula".to_string()),
                    cas: Some("100-00-5".to_string()),
                    ec: Some("214-480-6".to_string()),
                    molecular_weight: Some("1.5".to_string()),
                    molecular_weight_unit: Some("g/mol".to_string()),
                    boiling_point: Some("boiling_point".to_string()),
                    synonyms: Some(vec!["foo".to_string(), "bar".to_string()]),
                    symbols: Some(vec!["GHS01".to_string(), "GHS02".to_string()]),
                    signal: Some(vec!["danger".to_string()]),
                    hs: Some(vec!["H371".to_string(), "H372".to_string()]),
                    ps: Some(vec!["P390".to_string(), "P261".to_string()]),
                    twodpicture: Some("twodpicture".to_string()),
                },
                1,
                None,
            )
            .is_ok()
        );
    }
}
