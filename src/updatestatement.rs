use regex::Regex;
use rusqlite::Connection;
use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn update_ghs_statements(db_connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let hazard_statement_re = Regex::new(r"(?P<reference>H[0-9]+)[\t](?P<label>[^\t]+)[\t]")?;
    let precautionary_statement_re =
        Regex::new(r"(?P<reference>P[0-9+]+)[\t](?P<label>[^\t]+)[\t]")?;

    let lines = read_lines("./src/resources/ghscode_10.txt")?;
    for line in lines.map_while(Result::ok) {
        if let Some(captures) = hazard_statement_re.captures(&line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            db_connection.execute(
                "INSERT OR IGNORE hazardstatement (hazardstatement_label, hazardstatement_reference) VALUES (?1, ?2)",
                (&label, &reference),
            )?;
        } else if let Some(captures) = precautionary_statement_re.captures(&line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            db_connection.execute(
                "INSERT OR IGNORE precautionarystatement (precautionarystatement_label, precautionarystatement_reference) VALUES (?1, ?2)",
                (&label, &reference),
            )?;
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::init_db;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_update_ghs_statements() {
        init_logger();
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        assert!(update_ghs_statements(&db_connection).is_ok());
    }
}
