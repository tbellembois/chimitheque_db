use chimitheque_types::cenumber::Cenumber as CenumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Cenumber {
    Table,
    CenumberId,
    CenumberLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct CenumberWrapper(pub CenumberStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_cenumbers() {
        test_searchable(
            CenumberStruct {
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
