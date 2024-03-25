use crate::searchable::Searchable;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct EmpiricalformulaStruct {
    pub match_exact_search: bool,
    pub empiricalformula_id: u64,
    pub empiricalformula_label: String,
}

impl Searchable for EmpiricalformulaStruct {
    fn new(&self) -> Self {
        EmpiricalformulaStruct {
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
        String::from("empiricalformula")
    }

    fn get_id_field_name(&self) -> String {
        String::from("empiricalformula_id")
    }

    fn set_id_field(&mut self, id: u64) {
        self.empiricalformula_id = id;
    }

    fn get_text_field_name(&self) -> String {
        String::from("empiricalformula_label")
    }

    fn set_text_field(&mut self, text: &str) {
        self.empiricalformula_label = text.to_string();
    }

    fn get_id(&self) -> u64 {
        self.empiricalformula_id
    }

    fn get_text(&self) -> String {
        self.empiricalformula_label.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{init::init_db, searchable::get_many};
    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        // insert fake empiricalformulas.
        let _ = db_connection
            .execute(
                "INSERT INTO empiricalformula (empiricalformula_label) VALUES (?1)",
                [String::from("empiricalformula1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO empiricalformula (empiricalformula_label) VALUES (?1)",
                [String::from("aa empiricalformula1")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO empiricalformula (empiricalformula_label) VALUES (?1)",
                [String::from("empiricalformula2")],
            )
            .unwrap();
        let _ = db_connection
            .execute(
                "INSERT INTO empiricalformula (empiricalformula_label) VALUES (?1)",
                [String::from("empiricalformula3")],
            )
            .unwrap();

        db_connection
    }

    #[test]
    fn test_get_empiricalformulas() {
        init_logger();

        let db_connection = init_test_db();

        info!("testing ok result");
        assert!(get_many(
            EmpiricalformulaStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                ..Default::default()
            },
        )
        .is_ok());

        info!("testing filter search");
        let (empiricalformulas, count) = get_many(
            EmpiricalformulaStruct {
                ..Default::default()
            },
            &db_connection,
            RequestFilter {
                search: Some(String::from("empiricalformula1")),
                ..Default::default()
            },
        )
        .unwrap();

        // expected number of results.
        assert_eq!(count, 2);
        // expected exact match appears first.
        assert!(empiricalformulas[0].get_text().eq("empiricalformula1"))
    }
}
