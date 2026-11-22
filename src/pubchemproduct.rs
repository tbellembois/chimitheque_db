//! `PubChem` product import.
//!
//! A `PubChem` product is converted with `PubchemProduct::to_product`, which sets
//! its pick-list fields (symbols, hazard and precautionary statements, signal
//! word, molecular weight unit) by label only. [`resolve_pick_lists`] looks up
//! their ids; the product is then validated and saved like any other product.

use crate::searchable::parse;
use crate::{hazardstatement, precautionarystatement, unit};
use chimitheque_traits::searchable::Searchable;
use chimitheque_types::product::Product as ProductStruct;
use chimitheque_types::signalword::SignalWord;
use chimitheque_types::symbol::Symbol;
use rusqlite::Connection;
use std::fmt::{Display, Formatter};

/// A pick-list label of an imported product that is not in the database.
#[derive(Debug, PartialEq, Eq)]
pub enum ImportPubchemProductError {
    UnknownSymbol(String),
    UnknownHazardstatement(String),
    UnknownPrecautionarystatement(String),
    UnknownSignalword(String),
    UnknownMolecularWeightUnit(String),
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
        }
    }
}

impl std::error::Error for ImportPubchemProductError {}

/// Id and stored label of an item found by label.
type Found = (Option<u64>, String);

/// Looks up a searchable item by label, ignoring case, and returns its id
/// and stored label.
fn find_by_label(
    item: &(impl Searchable + std::fmt::Debug + Default + serde::Serialize),
    db_connection: &Connection,
    label: &str,
) -> Result<Option<Found>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(parse(item, db_connection, label)?.map(|found| (found.get_id(), found.get_text())))
}

/// Sets the ids of the pick-list fields of `product` from their labels:
/// symbols, hazard and precautionary statements (by reference), signal word
/// and molecular weight unit. Labels are matched ignoring case and replaced by
/// the stored ones. These items are never created by an import.
///
/// # Errors
///
/// Returns an `ImportPubchemProductError` for a label that is not in the
/// database, or a database error.
pub fn resolve_pick_lists(
    db_connection: &Connection,
    product: &mut ProductStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(symbols) = product.symbols.as_mut() {
        for symbol in symbols {
            let (id, label) =
                find_by_label(&Symbol::default(), db_connection, &symbol.symbol_label)?
                    .ok_or_else(|| {
                        ImportPubchemProductError::UnknownSymbol(symbol.symbol_label.clone())
                    })?;
            symbol.symbol_id = id;
            symbol.symbol_label = label;
        }
    }

    if let Some(hazard_statements) = product.hazard_statements.as_mut() {
        for hazard_statement in hazard_statements {
            *hazard_statement = hazardstatement::parse(
                db_connection,
                &hazard_statement.hazard_statement_reference,
            )?
            .ok_or_else(|| {
                ImportPubchemProductError::UnknownHazardstatement(
                    hazard_statement.hazard_statement_reference.clone(),
                )
            })?;
        }
    }

    if let Some(precautionary_statements) = product.precautionary_statements.as_mut() {
        for precautionary_statement in precautionary_statements {
            *precautionary_statement = precautionarystatement::parse(
                db_connection,
                &precautionary_statement.precautionary_statement_reference,
            )?
            .ok_or_else(|| {
                ImportPubchemProductError::UnknownPrecautionarystatement(
                    precautionary_statement
                        .precautionary_statement_reference
                        .clone(),
                )
            })?;
        }
    }

    if let Some(signal_word) = product.signal_word.as_mut() {
        let (id, label) = find_by_label(
            &SignalWord::default(),
            db_connection,
            &signal_word.signal_word_label,
        )?
        .ok_or_else(|| {
            ImportPubchemProductError::UnknownSignalword(signal_word.signal_word_label.clone())
        })?;
        signal_word.signal_word_id = id;
        signal_word.signal_word_label = label;
    }

    if let Some(unit_molecular_weight) = product.unit_molecular_weight.as_mut() {
        *unit_molecular_weight = unit::parse(db_connection, &unit_molecular_weight.unit_label)?
            .ok_or_else(|| {
                ImportPubchemProductError::UnknownMolecularWeightUnit(
                    unit_molecular_weight.unit_label.clone(),
                )
            })?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "pubchemproduct_tests.rs"]
mod pubchemproduct_tests;
