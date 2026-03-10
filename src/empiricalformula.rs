use chimitheque_types::empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum EmpiricalFormula {
    Table,
    EmpiricalFormulaId,
    EmpiricalFormulaLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct EmpiricalFormulaWrapper(pub EmpiricalFormulaStruct);
