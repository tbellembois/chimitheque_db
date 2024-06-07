use chimitheque_types::name::Name as NameStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Name {
    Table,
    NameId,
    NameLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct NameWrapper(pub NameStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_names() {
        test_searchable(
            NameStruct {
                ..Default::default()
            },
            vec!["name1", "aa name1", "bb nAmE1", "name2", "name3"],
            3,
            "name1",
        )
    }
}
