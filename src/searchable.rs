use chimitheque_traits::searchable::Searchable;
use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::{Connection, ToSql, params_from_iter};
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Write as _; // import without risk of name clashing

/// Returns the item whose text is `s`, ignoring the case, or `None`.
pub fn parse(
    item: &(impl Searchable + Debug + Default + Serialize),
    db_connection: &Connection,
    s: &str,
) -> Result<Option<impl Searchable + Serialize>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("s:{s:?}");

    // Select query statement.
    let select_query = format!(
        "SELECT {}, {} FROM {} WHERE {}==?1 COLLATE NOCASE",
        item.get_id_field_name(),
        item.get_text_field_name(),
        item.get_table_name(),
        item.get_text_field_name(),
    );

    // Perform select query.
    let mut stmt = db_connection.prepare(&select_query)?;
    let mayerr_query = stmt.query_row([s], |row| {
        let id: u64 = row.get_unwrap(0);
        let text: String = row.get_unwrap(1);

        Ok((id, text))
    });

    let (id, text) = match mayerr_query {
        Ok(r) => r,
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => return Ok(None),
            _ => return Err(Box::new(e)),
        },
    };

    // Build result.
    let mut new_item = item.create();
    new_item.set_id_field(id);
    new_item.set_text_field(text.as_str());

    Ok(Some(new_item))
}

/// Returns the items matching the filter, and their total count without limit
/// and offset.
///
/// `search` matches the texts containing it; an item whose text is exactly
/// `search` is flagged with `set_exact_search`. Otherwise `id` matches the id.
pub fn get_many<Var: Searchable + Debug + Default + Serialize>(
    item: &Var,
    db_connection: &Connection,
    filter: &RequestFilter, // Changed to take reference instead of cloning
) -> Result<(Vec<Var>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{filter:?}");

    // Parameters.
    let maybe_search = &filter.search;
    let maybe_id = &filter.id;

    let mut params: Vec<&dyn ToSql> = vec![];
    let wild_search: String;
    if let Some(search) = maybe_search {
        wild_search = format!("%{search}%");
        params = vec![&wild_search];
    } else if let Some(id) = maybe_id {
        params = vec![id];
    }

    // Select query statement.
    let mut select_query = format!(
        "SELECT {}, {} FROM {}",
        item.get_id_field_name(),
        item.get_text_field_name(),
        item.get_table_name()
    );

    if let Some(_search) = maybe_search {
        write!(
            &mut select_query,
            " WHERE {} LIKE (?1)",
            item.get_text_field_name()
        )
        .unwrap();
    } else if let Some(_id) = maybe_id {
        write!(
            &mut select_query,
            " WHERE {} = (?1)",
            item.get_id_field_name()
        )
        .unwrap();
    }

    write!(
        &mut select_query,
        " ORDER BY {} COLLATE NOCASE ASC",
        item.get_text_field_name()
    )
    .unwrap();

    if let Some(limit) = filter.limit {
        write!(&mut select_query, " LIMIT {limit}").unwrap();
    }

    if let Some(offset) = filter.offset {
        write!(&mut select_query, " OFFSET {offset}").unwrap();
    }

    debug!("select_query:{select_query:?}");

    let mut stmt = db_connection.prepare(&select_query)?;

    // Perform select query.
    let rows = stmt.query_map(params_from_iter(params.iter()), |row| {
        let mut new_item = item.create();

        let row_id: u64 = row.get(0)?;
        let row_text: String = row.get(1)?;

        debug!("row_id: {row_id}");
        debug!("row_text: {row_text}");

        new_item.set_id_field(row_id);
        new_item.set_text_field(&row_text);

        if let Some(search) = maybe_search
            && row_text.eq(search)
        {
            new_item.set_exact_search(true);
        }

        Ok(new_item)
    })?;

    // Build select result.
    let mut items = Vec::with_capacity(filter.limit.unwrap_or(100));

    for mayerr_item in rows {
        let item = mayerr_item?;

        if item.get_exact_search() {
            items.insert(0, item);
        } else {
            items.push(item);
        }
    }

    debug!("items:{items:#?}");

    // Count query statement.
    let mut count_query = format!(
        "SELECT COUNT(DISTINCT {}) FROM {}",
        item.get_id_field_name(),
        item.get_table_name()
    );

    if let Some(_search) = maybe_search {
        write!(
            &mut count_query,
            " WHERE {} LIKE (?1)",
            item.get_text_field_name()
        )
        .unwrap();
    } else if let Some(_id) = maybe_id {
        write!(
            &mut count_query,
            " WHERE {} = (?1)",
            item.get_id_field_name()
        )
        .unwrap();
    }

    debug!("count_query:{count_query:?}");

    // Perform count query.
    let mut stmt = db_connection.prepare(count_query.as_str())?;

    let mut rows = stmt.query(params_from_iter(params.iter()))?;

    let count: usize = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    debug!("count:{count}");

    Ok((items, count))
}

/// Updates the text of the item `item_id`, or without id creates an item with
/// the text, reusing an existing item with the same text. Returns its id.
pub fn create_update(
    item: &impl Searchable,
    item_id: Option<u64>,
    db_connection: &Connection,
    text: &str,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let last_insert_id: u64;

    if let Some(item_id) = item_id {
        let query = format!(
            "UPDATE {} SET {}=?1 WHERE {}=?2",
            item.get_table_name(),
            item.get_text_field_name(),
            item.get_id_field_name()
        );

        debug!("query:{query:?} 1:{text:?} 2:{item_id:?}");

        db_connection.execute(&query, [text.to_string(), item_id.to_string()])?;

        last_insert_id = item_id;
    } else {
        // Get or create: an item with the same (UNIQUE) text is reused. The no-op
        // update makes RETURNING give the existing id, which DO NOTHING would not.
        let query = format!(
            "INSERT INTO {table} ({text}) VALUES (?1) \
             ON CONFLICT ({text}) DO UPDATE SET {text} = excluded.{text} \
             RETURNING {id}",
            table = item.get_table_name(),
            text = item.get_text_field_name(),
            id = item.get_id_field_name()
        );

        debug!("{query:?}");

        last_insert_id = db_connection.query_row(&query, [text], |row| row.get(0))?;
    }

    Ok(last_insert_id)
}

/// Tests shared by the searchable types.
#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::{
        init::populate_db_with_base_data,
        searchable::{get_many, parse},
    };
    use chimitheque_types::requestfilter::RequestFilter;
    use log::info;
    use rusqlite::Connection;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Tests the search, parse and create functions of a searchable type.
    pub fn test_searchable(
        searchable: &(impl Searchable + Debug + Default + Serialize),
        fake_searchables: &[&str],
        test_search_count: usize,
        test_search_first_result: &str,
    ) {
        init_logger();

        let mut db_connection =
            Connection::open_in_memory().expect("Failed to open in-memory database");
        populate_db_with_base_data(&mut db_connection).unwrap();

        let table_name = searchable.get_table_name();
        let text_field_name = searchable.get_text_field_name();

        // Insert samples.
        for fake_searchable in fake_searchables {
            db_connection
                .execute(
                    &format!("INSERT INTO {table_name} ({text_field_name}) VALUES (?1)"),
                    [fake_searchable],
                )
                .unwrap();
        }

        info!("- testing total result for {table_name}");
        let request_filter = RequestFilter {
            ..Default::default()
        };
        let (searchables, total_count) =
            get_many(searchable, &db_connection, &request_filter).unwrap();
        assert_eq!(total_count, searchables.len());

        info!("- testing filter search for {table_name}");
        let request_filter = RequestFilter {
            ..Default::default()
        };
        let (searchables, count) = get_many(searchable, &db_connection, &request_filter).unwrap();
        // expected number of results.
        assert_eq!(count, test_search_count);
        // expected exact match appears first.
        assert!(searchables[0].get_text().eq(test_search_first_result));

        info!("- testing parse for {table_name}");
        let searchables = parse(searchable, &db_connection, fake_searchables[0]).unwrap();
        assert_eq!(
            searchables.unwrap().get_text(),
            fake_searchables[0].to_string()
        );

        let searchables = parse(searchable, &db_connection, "does not exist").unwrap();
        assert!(searchables.is_none());

        info!("- testing parse case insensitive for {table_name}");
        let mut randomized_chars: Vec<char> = Vec::new();
        let mut switch: bool = false;
        for c in fake_searchables[0].chars() {
            if switch {
                randomized_chars.push(c.to_ascii_lowercase());
            } else {
                randomized_chars.push(c.to_ascii_uppercase());
            }
            switch = !switch;
        }
        let randomized_string: String = randomized_chars.iter().collect();
        info!("-> generated {randomized_string}");

        let searchables = parse(searchable, &db_connection, &randomized_string)
            .unwrap()
            .unwrap();
        assert_eq!(searchables.get_text(), fake_searchables[0].to_string());
        assert!(searchables.get_id().is_some());

        info!("- testing count with limit for {table_name}");
        let request_filter = RequestFilter {
            ..Default::default()
        };
        let (searchables, count) = get_many(searchable, &db_connection, &request_filter).unwrap();
        assert_eq!(count, total_count);
        assert_eq!(searchables.len(), 5);

        info!("- testing create for {table_name}");

        let last_insert_id =
            create_update(searchable, None, &db_connection, "a non existing item").unwrap();
        assert!(last_insert_id > 0);

        // Creating an existing item reuses it instead of failing.
        let existing_id = parse(searchable, &db_connection, fake_searchables[0])
            .unwrap()
            .and_then(|existing| existing.get_id())
            .unwrap();
        let reused_id =
            create_update(searchable, None, &db_connection, fake_searchables[0]).unwrap();
        assert_eq!(reused_id, existing_id);
        let (_, count_after) = get_many(searchable, &db_connection, &request_filter).unwrap();
        assert_eq!(count_after, total_count + 1);
    }
}
