use chimitheque_types::physicalstate::Physicalstate as PhysicalstateStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Physicalstate {
    Table,
    PhysicalstateId,
    PhysicalstateLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct PhysicalstateWrapper(pub PhysicalstateStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_physicalstates() {
        test_searchable(
            PhysicalstateStruct {
                ..Default::default()
            },
            vec![
                "physicalstate1",
                "aa physicalstate1",
                "bb pHySiCaLsTaTe1",
                "physicalstate2",
                "physicalstate3",
            ],
            3,
            "physicalstate1",
        )
    }
}
