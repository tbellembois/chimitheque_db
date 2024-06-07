use chimitheque_types::casnumber::Casnumber as CasnumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Casnumber {
    Table,
    CasnumberId,
    CasnumberLabel,
    CasnumberCmr,
}

#[derive(Debug, Serialize, Default)]
pub struct CasnumberWrapper(pub CasnumberStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_casnumbers() {
        test_searchable(
            CasnumberStruct {
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
