use serde::Serialize;

use crate::basicsearch::Searchable;

#[derive(Debug, Serialize, Default)]
pub struct HazardstatementStruct {
    pub match_exact_search: bool,
    pub hazardstatement_id: u64,
    pub hazardstatement_label: String,
    pub hazardstatement_reference: String,
}

impl Searchable for HazardstatementStruct {
    fn new(&self) -> Self {
        HazardstatementStruct {
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
        String::from("hazardstatement")
    }

    fn get_id_field_name(&self) -> String {
        String::from("hazardstatement_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.hazardstatement_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("hazardstatement_reference")
    }

    fn set_text_field(&mut self, text: &str) {
        self.hazardstatement_reference = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.hazardstatement_id
    }

    fn get_text(&self) -> String {
        self.hazardstatement_reference.clone()
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

        // insert fake hazardstatements.
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement1"), String::from("hazardstatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("aa hazardstatement1"), String::from("aa hazardstatement1-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement2"), String::from("hazardstatement2-ref")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                [String::from("hazardstatement3"), String::from("hazardstatement3-ref")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_hazardstatements() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            HazardstatementStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (hazardstatements, count) = get_many(
            HazardstatementStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("hazardstatement1-ref")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(hazardstatements[0].get_text().eq("hazardstatement1-ref"))
    }
}
