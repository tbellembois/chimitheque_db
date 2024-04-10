use chimitheque_types::pubchemproduct::PubchemProduct;
use rusqlite::Connection;

use crate::searchable::{create, Searchable};
use crate::{name::NameStruct, searchable::parse};

pub fn create_product_from_pubchem(
    db_connection: &Connection,
    pubchem_product: PubchemProduct,
) -> Result<u64, Box<dyn std::error::Error>> {
    // Name.
    let name_id: Option<u64>;

    if let Some(name) = pubchem_product.name {
        let maybe_name = parse(
            &NameStruct {
                ..Default::default()
            },
            db_connection,
            &name,
        )?;

        name_id = match maybe_name {
            Some(name) => Some(name.get_id()),
            None => Some(create(
                &NameStruct {
                    ..Default::default()
                },
                db_connection,
                &name,
            )?),
        }
    }

    _ = name_id;

    Ok(15)
}
