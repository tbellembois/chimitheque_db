use chimitheque_types::category::Category as CategoryStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Category {
    Table,
    CategoryId,
    CategoryLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct CategoryWrapper(pub CategoryStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_categories() {
        test_searchable(
            CategoryStruct {
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
