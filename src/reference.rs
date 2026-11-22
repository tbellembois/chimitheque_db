//! Checks that the ids a request refers to exist.
//!
//! Saving a product, storage, store location, person or entity writes the ids
//! of other rows. An unknown id would make the save fail on a foreign key, so
//! callers list the references with the `*_references` functions of each
//! module and check them with [`find_missing_reference`] first.

use rusqlite::Connection;

/// An id written by a save: the request field it comes from, the table it
/// refers to, and the id. A `None` id is reported as missing: callers only
/// list the ids that must be set.
#[derive(Debug, PartialEq, Eq)]
pub struct Reference {
    /// Request field, e.g. `category`.
    pub field: &'static str,
    /// Referenced table, whose id column is `<table>_id`.
    pub table: &'static str,
    /// Referenced id.
    pub id: Option<u64>,
}

impl Reference {
    /// Creates a reference to `id` in `table`, from `field`.
    #[must_use]
    pub fn new(field: &'static str, table: &'static str, id: Option<u64>) -> Self {
        Self { field, table, id }
    }
}

/// Returns a message for the first reference without an id or whose id is
/// not in its table, or `None` when they all exist.
///
/// # Errors
///
/// Returns an error if a query fails.
pub fn find_missing_reference(
    db_connection: &Connection,
    references: &[Reference],
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    for reference in references {
        let Some(id) = reference.id else {
            return Ok(Some(format!("{} has no id", reference.field)));
        };

        // The table name is a static string from this crate, never from the
        // request, so it can be formatted into the query.
        let exists: bool = db_connection.query_row(
            &format!(
                "SELECT EXISTS (SELECT 1 FROM \"{0}\" WHERE \"{0}_id\" = ?1)",
                reference.table
            ),
            [id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(Some(format!("unknown {} id {id}", reference.field)));
        }
    }

    Ok(None)
}

#[cfg(test)]
#[path = "reference_tests.rs"]
mod reference_tests;
