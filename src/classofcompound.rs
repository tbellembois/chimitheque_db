use chimitheque_types::classofcompound::ClassOfCompound as ClassOfCompoundStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum ClassOfCompound {
    Table,
    ClassOfCompoundId,
    ClassOfCompoundLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct ClassOfCompoundWrapper(pub ClassOfCompoundStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_classes_of_compound() {
        test_searchable(
            ClassOfCompoundStruct {
                ..Default::default()
            },
            vec![
                "classesofcompound1",
                "aa classesofcompound1",
                "bb cLaSsOfCoMpOuNd1",
                "classesofcompound2",
                "classesofcompound3",
            ],
            2,
            "classesofcompound1",
        )
    }
}
