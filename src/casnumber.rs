use chimitheque_types::casnumber::CasNumber as CasNumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CasNumber {
    Table,
    CasNumberId,
    CasNumberLabel,
    CasNumberCmr,
}

#[derive(Debug, Serialize, Default)]
pub struct CasNumberWrapper(pub CasNumberStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_cas_numbers() {
        test_searchable(
            CasNumberStruct {
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
