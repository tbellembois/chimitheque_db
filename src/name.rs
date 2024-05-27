use chimitheque_types::name::Name;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct NameWrapper(pub Name);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_names() {
        test_searchable(
            Name {
                ..Default::default()
            },
            vec!["name1", "aa name1", "bb nAmE1", "name2", "name3"],
            3,
            "name1",
        )
    }
}
