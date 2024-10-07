use chimitheque_types::cenumber::CeNumber as CeNumberStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum CeNumber {
    Table,
    CeNumberId,
    CeNumberLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct CeNumberWrapper(pub CeNumberStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_ce_numbers() {
        test_searchable(
            CeNumberStruct {
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
