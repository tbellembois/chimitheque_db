use chimitheque_types::classofcompound::Classofcompound as ClassofcompoundStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Classofcompound {
    Table,
    ClassofcompoundId,
    ClassofcompoundLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct ClassofcompoundWrapper(pub ClassofcompoundStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_classesofcompound() {
        test_searchable(
            ClassofcompoundStruct {
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
