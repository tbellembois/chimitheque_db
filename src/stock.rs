use chimitheque_types::{
    stock::Stock, storelocation::StoreLocation as StoreLocationStruct, unit::Unit as UnitStruct,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{Alias, CaseStatement, Expr, Func, JoinType, Order, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{
    entity::Entity, permission::Permission, storage::Storage, storelocation::StoreLocation,
    unit::Unit,
};

#[derive(Debug, Serialize)]
pub struct StockWrapper(pub Stock);

impl From<&Row<'_>> for StockWrapper {
    fn from(row: &Row) -> Self {
        // Test if there is a parent unit..
        let maybe_parent_unit: Option<u64> = row.get_unwrap("parent_unit_id");

        Self({
            Stock {
                store_location: StoreLocationStruct {
                    store_location_id: row.get_unwrap("store_location_id"),
                    store_location_name: row.get_unwrap("store_location_name"),
                    store_location_full_path: row.get_unwrap("store_location_full_path"),
                    ..Default::default()
                },
                product: Default::default(),
                quantity: row.get("quantity").unwrap_or_default(), // quantity = 0 is returned null by sql.
                unit: maybe_parent_unit.map(|_| UnitStruct {
                    unit_id: row.get_unwrap("parent_unit_id"),
                    unit_label: row.get_unwrap("parent_unit_label"),
                    ..Default::default()
                }),
            }
        })
    }
}

// sample request:
// SELECT storelocation_id,storelocation_fullpath, SUM(storage_quantity * self_unit.unit_multiplier) AS quantity, parent_unit.unit_id, parent_unit.unit_label FROM storage
// LEFT JOIN storelocation ON storage.storelocation = storelocation.storelocation_id
// LEFT JOIN unit self_unit ON storage.unit_quantity = self_unit.unit_id
// LEFT JOIN unit parent_unit on parent_unit.unit_id = self_unit.unit
// WHERE storage.product = 6144
// GROUP BY self_unit.unit
pub fn compute_stock(
    db_connection: &Connection,
    product_id: u64,
    person_id: u64,
) -> Result<Vec<Stock>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("product_id:{:?}", product_id);
    debug!("person_id:{:?}", person_id);

    let (select_sql, select_values) = Query::select()
        .columns([
            StoreLocation::StoreLocationId,
            StoreLocation::StoreLocationName,
            StoreLocation::StoreLocationFullPath,
        ])
        .expr_as(
            Expr::col((Alias::new("parent_unit"), Unit::UnitId)),
            Alias::new("parent_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("parent_unit"), Unit::UnitLabel)),
            Alias::new("parent_unit_label"),
        )
        .expr_as(
            Expr::col((Alias::new("self_unit"), Unit::UnitId)),
            Alias::new("self_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("self_unit"), Unit::UnitLabel)),
            Alias::new("self_unit_label"),
        )
        .expr_as(
            Func::sum(
                CaseStatement::new()
                    .case(
                        Expr::col((Alias::new("self_unit"), Unit::UnitId)).is_null(),
                        Expr::col(Storage::StorageQuantity),
                    )
                    .finally(
                        Expr::col(Storage::StorageQuantity)
                            .mul(Expr::col((Alias::new("self_unit"), Unit::UnitMultiplier))),
                    ),
            ),
            Alias::new("quantity"),
        )
        .from(Storage::Table)
        .join(
            JoinType::InnerJoin,
            StoreLocation::Table,
            Expr::col((StoreLocation::Table, StoreLocation::StoreLocationId))
                .equals((Storage::Table, Storage::StoreLocation)),
        )
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("self_unit"),
            Expr::col((Alias::new("self_unit"), Unit::UnitId))
                .equals((Storage::Table, Storage::UnitQuantity)),
        )
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("parent_unit"),
            Expr::col((Alias::new("self_unit"), Unit::Unit))
                .equals((Alias::new("parent_unit"), Unit::UnitId)),
        )
        //
        // entity -> permissions
        //
        .join(
            // entity
            JoinType::InnerJoin,
            Entity::Table,
            Expr::col((StoreLocation::Table, StoreLocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        .join_as(
            JoinType::InnerJoin,
            Permission::Table,
            Alias::new("perm"),
            Expr::col((Alias::new("perm"), Alias::new("person")))
                .eq(person_id)
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_item")))
                        .is_in(["all", "storages"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity")))
                        .equals(Entity::EntityId)
                        .or(Expr::col(Entity::EntityId).is_null()) // products with no storages for non admins
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).eq(-1),
                        ),
                ),
        )
        .and_where(Expr::col((Storage::Table, Storage::Product)).eq(product_id))
        .and_where(Expr::col((Storage::Table, Storage::StorageArchive)).eq(false))
        .group_by_col(StoreLocation::StoreLocationId)
        .group_by_col((Alias::new("self_unit"), Unit::Unit))
        .order_by(StoreLocation::StoreLocationFullPath, Order::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform select query.
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let rows = stmt.query_map(&*select_values.as_params(), |row| {
        Ok(StockWrapper::from(row))
    })?;

    // Build select result.
    let mut stocks = Vec::new();
    for maybe_stock in rows {
        let stock = maybe_stock?;

        stocks.push(stock.0);
    }

    debug!("stocks: {:#?}", stocks);

    Ok(stocks)
}
