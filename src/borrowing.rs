use chimitheque_types::borrowing::Borrowing as BorrowingStruct;
use rusqlite::Row;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Borrowing {
    Table,
    BorrowingId,
    BorrowingComment,
    Person,
    Borrower,
    Storage,
}

#[derive(Debug, Serialize, Default)]
pub struct BorrowingWrapper(pub BorrowingStruct);

impl From<&Row<'_>> for BorrowingWrapper {
    fn from(row: &Row) -> Self {
        Self({
            BorrowingStruct {
                borrowing_id: row.get_unwrap("borrowing_id"),
                borrowing_comment: row.get_unwrap("borrowing_comment"),
                person: row.get_unwrap("person"),
                borrower: row.get_unwrap("borrower"),
                storage: row.get_unwrap("storage"),
            }
        })
    }
}
