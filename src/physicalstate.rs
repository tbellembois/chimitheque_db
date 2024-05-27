use chimitheque_types::physicalstate::Physicalstate;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct PhysicalstateWrapper(pub Physicalstate);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_physicalstates() {
        test_searchable(
            Physicalstate {
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
