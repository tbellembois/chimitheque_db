use log::debug;
use regex::Regex;
use rusqlite::Connection;

pub fn update_ghs_statements(
    db_connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_transaction = db_connection.transaction()?;

    let hazard_statement_re =
        Regex::new(r"(?P<reference>(EU){0,1}H[0-9]+)(\t)(?P<label>[^\t]+)(\t)")?;
    let precautionary_statement_re =
        Regex::new(r"(?P<reference>P[0-9+]+)(\t)(?P<label>[^\t]+)(\t)")?;

    let file = include_str!("resources/ghscode_10.txt");
    for line in file.lines() {
        debug!("{:?}", line);

        if let Some(captures) = hazard_statement_re.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("{reference}: {label}");

            db_transaction.execute(
            "INSERT INTO hazard_statement (hazard_statement_label, hazard_statement_reference) VALUES (?1, ?2) ON CONFLICT(hazard_statement_reference) DO UPDATE SET hazard_statement_label=?1;",
            (&label, &reference),
            )?;
        } else if let Some(captures) = precautionary_statement_re.captures(line) {
            let reference = captures.name("reference").unwrap().as_str();
            let label = captures.name("label").unwrap().as_str();

            debug!("{reference}: {label}");

            db_transaction.execute(
            "INSERT INTO precautionary_statement (precautionary_statement_label, precautionary_statement_reference) VALUES (?1, ?2) ON CONFLICT(precautionary_statement_reference) DO UPDATE SET precautionary_statement_label=?1;",
            (&label, &reference),
            )?;
        };
    }

    db_transaction.commit()?;

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

        assert!(update_ghs_statements(&mut db_connection).is_ok());
    }
}
