use chimitheque_types::empiricalformula::Empiricalformula;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct EmpiricalformulaWrapper(pub Empiricalformula);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_empiricalformulas() {
        test_searchable(
            Empiricalformula {
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
