use std::fmt::Debug;

use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::Connection;
use serde::Serialize;

pub trait Searchable {
    fn new(&self) -> Self;

    fn set_exact_search(&mut self, match_exact_search: bool);
    fn get_exact_search(&self) -> bool;

    fn get_table_name(&self) -> String;

    fn get_id_field_name(&self) -> String;
    fn get_text_field_name(&self) -> String;

    fn set_id_field(&mut self, id: u64);
    fn set_text_field(&mut self, text: &str);

    fn get_id(&self) -> u64;
    fn get_text(&self) -> String;
}

pub fn get_many(
    item: impl Searchable + Debug + Default + Serialize,
    db_connection: &Connection,
    filter: RequestFilter,
) -> Result<(Vec<impl Searchable + Serialize>, usize), Box<dyn std::error::Error>> {
    debug!("filter:{:?}", filter);

    // Select query statement.
    let mut select_query = format!(
        "SELECT {}, {} FROM {}",
        item.get_id_field_name(),
        item.get_text_field_name(),
        item.get_table_name()
    );

    if let Some(search) = filter.search.clone() {
        select_query.push_str(&format!(
            " WHERE {} LIKE '%{}%'",
            item.get_text_field_name(),
            search
        ))
    }

    if let Some(limit) = filter.limit {
        select_query.push_str(&format!(" LIMIT {}", limit))
    }

    if let Some(offset) = filter.offset {
        select_query.push_str(&format!(" OFFSET {}", offset))
    }

    // Count query statement.
    let mut count_query = format!(
        "SELECT COUNT(DISTINCT {}) FROM {}",
        item.get_id_field_name(),
        item.get_table_name()
    );

    if let Some(search) = filter.search.clone() {
        count_query.push_str(&format!(
            " WHERE {} LIKE '%{}%'",
            item.get_text_field_name(),
            search
        ))
    }

    // Perform count query.
    let mut stmt = db_connection.prepare(count_query.as_str())?;
    let mut rows = stmt.query(())?;
    let count: usize = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_query)?;
    let rows = stmt.query_map((), |row| {
        let mut new_item = item.new();

        let row_id: u64 = row.get(0)?;
        let row_text: String = row.get(1)?;

        debug!("row_id: {row_id}");
        debug!("row_text: {row_text}");

        new_item.set_id_field(row_id);
        new_item.set_text_field(&row_text);

        if filter.search.is_some() && row_text.eq(&filter.search.clone().unwrap()) {
            new_item.set_exact_search(true);
        }

        Ok(new_item)
    })?;

    // Build select result.
    let mut items = Vec::new();

    for maybe_item in rows {
        let item = maybe_item?;

        if item.get_exact_search() {
            items.insert(0, item)
        } else {
            items.push(item);
        }
    }

    debug!("items:{:#?}", items);
    debug!("items:{}", count);

    Ok((items, count))
}
