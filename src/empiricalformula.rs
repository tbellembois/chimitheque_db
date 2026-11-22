use chimitheque_types::empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `empirical_formula` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum EmpiricalFormula {
    /// The table.
    Table,
    /// The `empirical_formula_id` column.
    EmpiricalFormulaId,
    /// The `empirical_formula_label` column.
    EmpiricalFormulaLabel,
}

/// A empirical formula read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct EmpiricalFormulaWrapper(pub EmpiricalFormulaStruct);
