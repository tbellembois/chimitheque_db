use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct PhysicalstateStruct {
    pub match_exact_search: bool,
    pub physicalstate_id: u64,
    pub physicalstate_label: String,
}

impl Searchable for PhysicalstateStruct {
    fn new(&self) -> Self {
        PhysicalstateStruct {
            ..Default::default()
        }
    }

    fn set_exact_search(&mut self, match_exact_search: bool) {
        self.match_exact_search = match_exact_search;
    }

    fn get_exact_search(&self) -> bool {
        self.match_exact_search
    }

    fn get_table_name(&self) -> String {
        String::from("physicalstate")
    }

    fn get_id_field_name(&self) -> String {
        String::from("physicalstate_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.physicalstate_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("physicalstate_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.physicalstate_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.physicalstate_id
    }

    fn get_text(&self) -> String {
        self.physicalstate_label.clone()
    }
}

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
