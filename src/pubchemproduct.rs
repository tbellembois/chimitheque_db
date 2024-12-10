use crate::hazardstatement;
use crate::product::Product;
use crate::producthazardstatements::Producthazardstatements;
use crate::productprecautionarystatements::Productprecautionarystatements;
use crate::productsymbols::Productsymbols;
use crate::searchable::create;
use crate::searchable::parse;
use crate::{precautionarystatement, unit};
use chimitheque_traits::searchable::Searchable;
use chimitheque_types::casnumber::CasNumber;
use chimitheque_types::cenumber::CeNumber;
use chimitheque_types::empiricalformula::EmpiricalFormula;
use chimitheque_types::name::Name;
use chimitheque_types::pubchemproduct::PubchemProduct;
use chimitheque_types::signalword::SignalWord;
use chimitheque_types::symbol::Symbol;
use chimitheque_utils::casnumber::is_cas_number;
use chimitheque_utils::cenumber::is_ce_number;
use chimitheque_utils::formula::sort_empirical_formula;
use log::debug;
use rusqlite::Connection;
use sea_query::Expr;
use sea_query::{Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use sea_query_rusqlite::RusqliteValues;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum ImportPubchemProductError {
    UnknownSymbol(String),
    UnknownHazardstatement(String),
    UnknownPrecautionarystatement(String),
    UnknownSignalword(String),
    UnknownMolecularWeightUnit(String),
    InvalidCasnumber(String),
    InvalidEcnumber(String),
    EmptyName,
}

impl Display for ImportPubchemProductError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ImportPubchemProductError::UnknownSymbol(s) => write!(f, "unknown symbol {s}"),
            ImportPubchemProductError::UnknownHazardstatement(s) => {
                write!(f, "unknown hazard statement {s}")
            }
            ImportPubchemProductError::UnknownPrecautionarystatement(s) => {
                write!(f, "unknown precautionary statement {s}")
            }
            ImportPubchemProductError::UnknownSignalword(s) => {
                write!(f, "unknown signal word {s}")
            }
            ImportPubchemProductError::UnknownMolecularWeightUnit(s) => {
                write!(f, "unknown molecular weight unit {s}")
            }
            ImportPubchemProductError::InvalidCasnumber(s) => {
                write!(f, "invalid cas number {s}")
            }
            ImportPubchemProductError::InvalidEcnumber(s) => {
                write!(f, "invalid ec number {s}")
            }
            ImportPubchemProductError::EmptyName => {
                write!(f, "empty name")
            }
        }
    }
}

impl std::error::Error for ImportPubchemProductError {}

pub fn create_update_product_from_pubchem(
    db_connection: &Connection,
    pubchem_product: PubchemProduct,
    person_id: u64,
    product_id: Option<u64>,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("pubchem_product: {:#?}", pubchem_product);

    // Mandatory name.
    let name_id: u64;

    if let Some(name_text) = pubchem_product.name {
        let maybe_name = parse(
            &Name {
                ..Default::default()
            },
            db_connection,
            &name_text,
        )?;

        name_id = match maybe_name {
            Some(name) => name.get_id(),
            None => create(
                &Name {
                    ..Default::default()
                },
                db_connection,
                &name_text.to_uppercase(),
            )?,
        }
    } else {
        return Err(Box::new(ImportPubchemProductError::EmptyName));
    }

    // Update request: list of (columns, values) pairs to insert in the product table.
    let mut columns_values = vec![
        (Product::Name, SimpleExpr::Value(name_id.into())),
        (Product::Person, SimpleExpr::Value(person_id.into())),
    ];

    // Create request: list of columns and values to insert in the product table.
    let mut columns = vec![Product::Name, Product::Person];
    let mut values = vec![
        SimpleExpr::Value(name_id.into()),
        SimpleExpr::Value(person_id.into()),
    ];

    // Basic fields.
    if let Some(inchi) = pubchem_product.inchi {
        columns.push(Product::ProductInchi);
        values.push(SimpleExpr::Value(inchi.clone().into()));

        columns_values.push((Product::ProductInchi, SimpleExpr::Value(inchi.into())));
    }

    if let Some(inchi_key) = pubchem_product.inchi_key {
        columns.push(Product::ProductInchikey);
        values.push(SimpleExpr::Value(inchi_key.clone().into()));

        columns_values.push((
            Product::ProductInchikey,
            SimpleExpr::Value(inchi_key.into()),
        ));
    }

    if let Some(canonical_smiles) = pubchem_product.canonical_smiles {
        columns.push(Product::ProductCanonicalSmiles);
        values.push(SimpleExpr::Value(canonical_smiles.clone().into()));

        columns_values.push((
            Product::ProductCanonicalSmiles,
            SimpleExpr::Value(canonical_smiles.into()),
        ));
    }

    if let Some(molecular_weight) = pubchem_product.molecular_weight {
        columns.push(Product::ProductMolecularWeight);
        values.push(SimpleExpr::Value(molecular_weight.clone().into()));

        columns_values.push((
            Product::ProductMolecularWeight,
            SimpleExpr::Value(molecular_weight.into()),
        ));
    }

    if let Some(twodpicture) = pubchem_product.twodpicture {
        columns.push(Product::ProductTwodFormula);
        values.push(SimpleExpr::Value(twodpicture.clone().into()));

        columns_values.push((
            Product::ProductTwodFormula,
            SimpleExpr::Value(twodpicture.into()),
        ));
    }

    // Molecular weight unit.
    if let Some(molecularweight_unit_text) = pubchem_product.molecular_weight_unit {
        let maybe_molecularweight_unit = unit::parse(db_connection, &molecularweight_unit_text)?;

        let molecularweight_unit_id = match maybe_molecularweight_unit {
            Some(molecular_weight) => molecular_weight.unit_id,
            None => {
                return Err(Box::new(
                    ImportPubchemProductError::UnknownMolecularWeightUnit(
                        molecularweight_unit_text,
                    ),
                ))
            }
        };

        columns.push(Product::UnitMolecularWeight);
        values.push(SimpleExpr::Value(molecularweight_unit_id.into()));

        columns_values.push((
            Product::UnitMolecularWeight,
            SimpleExpr::Value(molecularweight_unit_id.into()),
        ));
    }

    // Cas number.
    if let Some(casnumber_text) = pubchem_product.cas {
        if !is_cas_number(&casnumber_text)? {
            return Err(Box::new(ImportPubchemProductError::InvalidCasnumber(
                casnumber_text,
            )));
        };

        let maybe_casnumber = parse(
            &CasNumber {
                ..Default::default()
            },
            db_connection,
            &casnumber_text,
        )?;

        let casnumber_id = match maybe_casnumber {
            Some(casnumber) => Some(casnumber.get_id()),
            None => Some(create(
                &CasNumber {
                    ..Default::default()
                },
                db_connection,
                &casnumber_text,
            )?),
        };

        columns.push(Product::CasNumber);
        values.push(SimpleExpr::Value(casnumber_id.into()));

        columns_values.push((Product::CasNumber, SimpleExpr::Value(casnumber_id.into())));
    }

    // Ec number.
    if let Some(ecnumber_text) = pubchem_product.ec {
        if !is_ce_number(&ecnumber_text)? {
            return Err(Box::new(ImportPubchemProductError::InvalidEcnumber(
                ecnumber_text,
            )));
        };

        let maybe_ecnumber = parse(
            &CeNumber {
                ..Default::default()
            },
            db_connection,
            &ecnumber_text,
        )?;

        let ecnumber_id = match maybe_ecnumber {
            Some(ecnumber) => Some(ecnumber.get_id()),
            None => Some(create(
                &CeNumber {
                    ..Default::default()
                },
                db_connection,
                &ecnumber_text,
            )?),
        };

        columns.push(Product::CeNumber);
        values.push(SimpleExpr::Value(ecnumber_id.into()));

        columns_values.push((Product::CeNumber, SimpleExpr::Value(ecnumber_id.into())));
    }

    // Empirical formula.
    if let Some(empiricalformula_text) = pubchem_product.molecular_formula {
        let sorted_empiricalformula = sort_empirical_formula(&empiricalformula_text)?;

        let maybe_empiricalformula = parse(
            &EmpiricalFormula {
                ..Default::default()
            },
            db_connection,
            &sorted_empiricalformula,
        )?;

        let empiricalformula_id = match maybe_empiricalformula {
            Some(empiricalformula) => Some(empiricalformula.get_id()),
            None => Some(create(
                &EmpiricalFormula {
                    ..Default::default()
                },
                db_connection,
                &empiricalformula_text,
            )?),
        };

        columns.push(Product::EmpiricalFormula);
        values.push(SimpleExpr::Value(empiricalformula_id.into()));

        columns_values.push((
            Product::EmpiricalFormula,
            SimpleExpr::Value(empiricalformula_id.into()),
        ));
    }

    // Signal word.
    if let Some(signals_text) = pubchem_product.signal {
        if let Some(signalword) = signals_text.first() {
            let maybe_signalword = parse(
                &SignalWord {
                    ..Default::default()
                },
                db_connection,
                signalword,
            )?;

            let signalword_id = match maybe_signalword {
                Some(signalword) => signalword.get_id(),
                None => {
                    return Err(Box::new(ImportPubchemProductError::UnknownSignalword(
                        signalword.to_string(),
                    )))
                }
            };

            columns.push(Product::SignalWord);
            values.push(SimpleExpr::Value(signalword_id.into()));

            columns_values.push((Product::SignalWord, SimpleExpr::Value(signalword_id.into())));
        }
    }

    // Symbols.
    let mut symbol_ids: Vec<u64> = Vec::new();

    if let Some(symbols_text) = pubchem_product.symbols {
        for symbol in symbols_text {
            let maybe_symbol = parse(
                &Symbol {
                    ..Default::default()
                },
                db_connection,
                &symbol,
            )?;

            let symbol_id = match maybe_symbol {
                Some(symbol) => symbol.get_id(),
                None => return Err(Box::new(ImportPubchemProductError::UnknownSymbol(symbol))),
            };

            symbol_ids.push(symbol_id);
        }
    }

    // Hazard statements.
    let mut hazardstatement_ids: Vec<u64> = Vec::new();

    if let Some(hazardstatements_text) = pubchem_product.hs {
        for hazardstatement in hazardstatements_text {
            let maybe_hazardstatement = hazardstatement::parse(db_connection, &hazardstatement)?;

            let hazardstatement_id = match maybe_hazardstatement {
                Some(hazardstatement) => hazardstatement.hazard_statement_id,
                None => {
                    return Err(Box::new(ImportPubchemProductError::UnknownHazardstatement(
                        hazardstatement,
                    )))
                }
            };

            hazardstatement_ids.push(hazardstatement_id);
        }
    }

    // Precautionary statements.
    let mut precautionary_statement_ids: Vec<u64> = Vec::new();

    if let Some(precautionarystatements_text) = pubchem_product.ps {
        for precautionarystatement in precautionarystatements_text {
            let maybe_precautionarystatement =
                precautionarystatement::parse(db_connection, &precautionarystatement)?;

            let precautionary_statement_id = match maybe_precautionarystatement {
                Some(precautionarystatement) => precautionarystatement.precautionary_statement_id,
                None => {
                    return Err(Box::new(
                        ImportPubchemProductError::UnknownPrecautionarystatement(
                            precautionarystatement,
                        ),
                    ))
                }
            };

            precautionary_statement_ids.push(precautionary_statement_id);
        }
    }

    let sql_query: String;
    let mut sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(product_id) = product_id {
        // Update query.
        (sql_query, sql_values) = Query::update()
            .table(Product::Table)
            .values(columns_values)
            .and_where(Expr::col(Product::ProductId).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(Product::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(product_id) = product_id {
        last_insert_update_id = product_id;
    } else {
        last_insert_update_id = db_connection.last_insert_rowid().try_into()?;
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);
    debug!("last_insert_update_id: {}", last_insert_update_id);

    // Insert symbols.
    for symbol_id in symbol_ids {
        let sql_query = Query::insert()
            .into_table(Productsymbols::Table)
            .columns([
                Productsymbols::ProductsymbolsProductId,
                Productsymbols::ProductsymbolsSymbolId,
            ])
            .values([last_insert_update_id.into(), symbol_id.into()])?
            .to_string(SqliteQueryBuilder);

        _ = db_connection.execute(&sql_query, ())?;
    }

    // Insert hazard statements.
    for hazardstatement_id in hazardstatement_ids {
        let sql_query = Query::insert()
            .into_table(Producthazardstatements::Table)
            .columns([
                Producthazardstatements::ProducthazardstatementsProductId,
                Producthazardstatements::ProducthazardstatementsHazardStatementId,
            ])
            .values([last_insert_update_id.into(), hazardstatement_id.into()])?
            .to_string(SqliteQueryBuilder);

        _ = db_connection.execute(&sql_query, ())?;
    }

    // Insert precautionary statements.
    for precautionary_statement_id in precautionary_statement_ids {
        let sql_query = Query::insert()
            .into_table(Productprecautionarystatements::Table)
            .columns([
                Productprecautionarystatements::ProductprecautionarystatementsProductId,
                Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId,
            ])
            .values([last_insert_update_id.into(), precautionary_statement_id.into()])?
            .to_string(SqliteQueryBuilder);

        _ = db_connection.execute(&sql_query, ())?;
    }

    Ok(last_insert_update_id)
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;
    use crate::init::init_db;
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        db_connection
    }

    #[test]
    fn test_create_product_from_pubchem() {
        init_logger();

        let mut db_connection = init_test_db();

        info!("testing create product from pubchem");
        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                name: Some("aspirin".to_string()),
                ..Default::default()
            },
            1,
            None,
        )
        .is_ok_and(|id| id > 0));

        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                ..Default::default()
            },
            1,
            None,
        )
        .is_err_and(|e| e.to_string().eq("empty name")));

        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                name: Some("aspirin".to_string()),
                hs: Some(vec!["foo".to_string()]),
                ..Default::default()
            },
            1,
            None,
        )
        .is_err_and(|e| e.to_string().eq("unknown hazard statement foo")));

        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                name: Some("aspirin".to_string()),
                ps: Some(vec!["foo".to_string()]),
                ..Default::default()
            },
            1,
            None,
        )
        .is_err_and(|e| e.to_string().eq("unknown precautionary statement foo")));

        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                name: Some("aspirin".to_string()),
                signal: Some(vec!["foo".to_string()]),
                ..Default::default()
            },
            1,
            None,
        )
        .is_err_and(|e| e.to_string().eq("unknown signal word foo")));

        assert!(create_update_product_from_pubchem(
            &db_connection,
            PubchemProduct {
                name: Some("aspirin".to_string()),
                molecular_weight_unit: Some("foo".to_string()),
                ..Default::default()
            },
            1,
            None,
        )
        .is_err_and(|e| e.to_string().eq("unknown molecular weight unit foo")));

        assert!(create_update_product_from_pubchem(
            &db_connection,
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
                hs: Some(vec!["EUH209A".to_string(), "EUH209".to_string()]),
                ps: Some(vec!["P390".to_string(), "P261".to_string()]),
                twodpicture: Some("twodpicture".to_string()),
            },
            1,
            None,
        )
        .is_ok());
    }
}
