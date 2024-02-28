use serde::Serialize;

use crate::searchable::Searchable;

#[derive(Debug, Serialize, Default)]
pub struct PrecautionarystatementStruct {
    pub match_exact_search: bool,
    pub precautionarystatement_id: u64,
    pub precautionarystatement_label: String,
    pub precautionarystatement_reference: String,
}

impl Searchable for PrecautionarystatementStruct {
    fn new(&self) -> Self {
        PrecautionarystatementStruct {
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
        String::from("precautionarystatement")
    }

    fn get_id_field_name(&self) -> String {
        String::from("precautionarystatement_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.precautionarystatement_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("precautionarystatement_reference")
    }

    fn set_text_field(&mut self, text: &str) {
        self.precautionarystatement_reference = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.precautionarystatement_id
    }

    fn get_text(&self) -> String {
        self.precautionarystatement_reference.clone()
    }
}

#[cfg(test)]
mod tests {

    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    use crate::{init::init_db, searchable::get_many};

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        // insert fake precautionarystatements.
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement1"), String::from("precautionarystatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("aa precautionarystatement1"), String::from("aa precautionarystatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement2"), String::from("precautionarystatement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                [String::from("precautionarystatement3"), String::from("precautionarystatement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_precautionarystatements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            PrecautionarystatementStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (precautionarystatements, count) = get_many(
            PrecautionarystatementStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("precautionarystatement1-ref")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(precautionarystatements[0]
            .get_text()
            .eq("precautionarystatement1-ref"))
    }
}
