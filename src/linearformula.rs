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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_linear_formulas() {
        test_searchable(
            LinearFormulaStruct {
                ..Default::default()
            },
            vec![
                "linearformula1",
                "aa linearformula1",
                "bb lInEaRFoRmULa1",
                "linearformula2",
                "linearformula3",
            ],
            3,
            "linearformula1",
        )
    }
}
