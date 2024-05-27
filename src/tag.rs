use chimitheque_types::tag::Tag;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct TagWrapper(pub Tag);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_tags() {
        test_searchable(
            Tag {
                ..Default::default()
            },
            vec!["tag1", "aa tag1", "bb tAg1", "tag2", "tag3"],
            3,
            "tag1",
        )
    }
}
