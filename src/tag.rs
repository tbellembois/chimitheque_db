use chimitheque_types::tag::Tag as TagStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Tag {
    Table,
    TagId,
    TagLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct TagWrapper(pub TagStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_tags() {
        test_searchable(
            TagStruct {
                ..Default::default()
            },
            vec!["tag1", "aa tag1", "bb tAg1", "tag2", "tag3"],
            3,
            "tag1",
        )
    }
}
