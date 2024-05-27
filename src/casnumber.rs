use chimitheque_types::casnumber::Casnumber;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CasnumberWrapper(pub Casnumber);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_casnumbers() {
        test_searchable(
            Casnumber {
                ..Default::default()
            },
            vec![
                "casnumber1",
                "aa casnumber1",
                "bb cAsNuMbEr1",
                "casnumber2",
                "casnumber3",
            ],
            3,
            "casnumber1",
        )
    }
}
