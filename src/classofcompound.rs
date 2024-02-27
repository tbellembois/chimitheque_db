use serde::Serialize;

use crate::basicsearch::Searchable;

#[derive(Debug, Serialize, Default)]
pub struct ClassofcompoundStruct {
    pub(crate) match_exact_search: bool,
    pub(crate) classofcompound_id: u64,
    pub(crate) classofcompound_label: String,
}

impl Searchable for ClassofcompoundStruct {
    fn new(&self) -> Self {
        ClassofcompoundStruct {
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
        String::from("classofcompound")
    }

    fn get_id_field_name(&self) -> String {
        String::from("classofcompound_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.classofcompound_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("classofcompound_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.classofcompound_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.classofcompound_id
    }

    fn get_text(&self) -> String {
        self.classofcompound_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    use crate::{basicsearch::get_many, init::init_db};

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        // insert fake classofcompounds.
        let _ = db_connection
            .execute(
                "INSERT INTO classofcompound (classofcompound_label) VALUES (?1)",
                [String::from("classofcompound1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO classofcompound (classofcompound_label) VALUES (?1)",
                [String::from("aa classofcompound1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO classofcompound (classofcompound_label) VALUES (?1)",
                [String::from("classofcompound2")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO classofcompound (classofcompound_label) VALUES (?1)",
                [String::from("classofcompound3")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_classofcompounds() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            ClassofcompoundStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (classofcompounds, count) = get_many(
            ClassofcompoundStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("classofcompound1")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(classofcompounds[0].get_text().eq("classofcompound1"))
    }
}
