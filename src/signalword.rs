use serde::Serialize;

use crate::searchable::Searchable;

#[derive(Debug, Serialize, Default)]
pub struct SignalwordStruct {
    pub match_exact_search: bool,
    pub signalword_id: u64,
    pub signalword_label: String,
}

impl Searchable for SignalwordStruct {
    fn new(&self) -> Self {
        SignalwordStruct {
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
        String::from("signalword")
    }

    fn get_id_field_name(&self) -> String {
        String::from("signalword_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.signalword_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("signalword_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.signalword_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.signalword_id
    }

    fn get_text(&self) -> String {
        self.signalword_label.clone()
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

        // insert fake signalwords.
        let _ = db_connection
            .execute(
                "INSERT INTO signalword (signalword_label) VALUES (?1)",
                [String::from("signalword1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO signalword (signalword_label) VALUES (?1)",
                [String::from("aa signalword1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO signalword (signalword_label) VALUES (?1)",
                [String::from("signalword2")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO signalword (signalword_label) VALUES (?1)",
                [String::from("signalword3")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_signalwords() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            SignalwordStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (signalwords, count) = get_many(
            SignalwordStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("signalword1")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(signalwords[0].get_text().eq("signalword1"))
    }
}
