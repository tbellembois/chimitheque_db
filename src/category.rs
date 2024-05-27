use chimitheque_types::category::Category;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct CategoryWrapper(pub Category);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_categories() {
        test_searchable(
            Category {
                ..Default::default()
            },
            vec![
                "category1",
                "aa category1",
                "bb cAtEgOrY1",
                "category2",
                "category3",
            ],
            3,
            "category1",
        )
    }
}
