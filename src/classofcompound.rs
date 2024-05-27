use chimitheque_types::classofcompound::Classofcompound;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ClassofcompoundWrapper(pub Classofcompound);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_classesofcompound() {
        test_searchable(
            Classofcompound {
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
