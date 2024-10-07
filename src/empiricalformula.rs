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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_empirical_formulas() {
        test_searchable(
            EmpiricalFormulaStruct {
                ..Default::default()
            },
            vec![
                "empiricalformula1",
                "aa empiricalformula1",
                "bb eMpIrIcAlFoRmULa1",
                "empiricalformula2",
                "empiricalformula3",
            ],
            3,
            "empiricalformula1",
        )
    }
}
