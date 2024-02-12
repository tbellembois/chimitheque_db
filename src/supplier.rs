use chimitheque_types::requestfilter::RequestFilter;
use log::debug;
use rusqlite::Connection;
use sea_query::Iden;

#[derive(Iden)]
enum Supplier {
    Table,
    SupplierId,
    SupplierLabel,
}

pub fn get_suppliers(
    db_connection: Connection,
    filter: RequestFilter,
) -> Result<(Vec<Supplier>, usize), String> {
    debug!("filter:{:?}", filter);
}
