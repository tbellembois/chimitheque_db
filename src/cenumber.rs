use serde::Serialize;

use crate::basicsearch::Searchable;

#[derive(Debug, Serialize, Default)]
pub struct CenumberStruct {
    pub(crate) match_exact_search: bool,
    pub(crate) cenumber_id: u64,
    pub(crate) cenumber_label: String,
}

impl Searchable for CenumberStruct {
    fn new(&self) -> Self {
        CenumberStruct {
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
        String::from("cenumber")
    }

    fn get_id_field_name(&self) -> String {
        String::from("cenumber_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.cenumber_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("cenumber_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.cenumber_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.cenumber_id
    }

    fn get_text(&self) -> String {
        self.cenumber_label.clone()
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

        // insert fake cenumbers.
        let _ = db_connection
            .execute(
                "INSERT INTO cenumber (cenumber_label) VALUES (?1)",
                [String::from("cenumber1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO cenumber (cenumber_label) VALUES (?1)",
                [String::from("aa cenumber1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO cenumber (cenumber_label) VALUES (?1)",
                [String::from("cenumber2")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO cenumber (cenumber_label) VALUES (?1)",
                [String::from("cenumber3")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_cenumbers() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            CenumberStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (cenumbers, count) = get_many(
            CenumberStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("cenumber1")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(cenumbers[0].get_text().eq("cenumber1"))
    }
}
