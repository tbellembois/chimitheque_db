use chimitheque_types::cenumber::Cenumber;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CenumberWrapper(pub Cenumber);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_cenumbers() {
        test_searchable(
            Cenumber {
                ..Default::default()
            },
            vec![
                "cenumber1",
                "aa cenumber1",
                "bb cENuMbEr1",
                "cenumber2",
                "cenumber3",
            ],
            3,
            "cenumber1",
        )
    }
}
