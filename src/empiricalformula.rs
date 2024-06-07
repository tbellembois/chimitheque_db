use chimitheque_types::empiricalformula::Empiricalformula as EmpiricalformulaStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Empiricalformula {
    Table,
    EmpiricalformulaId,
    EmpiricalformulaLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct EmpiricalformulaWrapper(pub EmpiricalformulaStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_empiricalformulas() {
        test_searchable(
            EmpiricalformulaStruct {
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
