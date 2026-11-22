use chimitheque_types::linearformula::LinearFormula as LinearFormulaStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `linear_formula` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum LinearFormula {
    /// The table.
    Table,
    /// The `linear_formula_id` column.
    LinearFormulaId,
    /// The `linear_formula_label` column.
    LinearFormulaLabel,
}

/// A linear formula read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct LinearFormulaWrapper(pub LinearFormulaStruct);
