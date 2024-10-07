use chimitheque_types::physicalstate::PhysicalState as PhysicalStateStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum PhysicalState {
    Table,
    PhysicalStateId,
    PhysicalStateLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct PhysicalStateWrapper(pub PhysicalStateStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_physical_states() {
        test_searchable(
            PhysicalStateStruct {
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
