use chimitheque_types::linearformula::Linearformula;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct LinearformulaWrapper(pub Linearformula);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_linearformulas() {
        test_searchable(
            Linearformula {
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
