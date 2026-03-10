use chimitheque_types::linearformula::LinearFormula as LinearFormulaStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum LinearFormula {
    Table,
    LinearFormulaId,
    LinearFormulaLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct LinearFormulaWrapper(pub LinearFormulaStruct);
