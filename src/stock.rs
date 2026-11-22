use chimitheque_types::{
    product::Product, stock::Stock, storelocation::StoreLocation as StoreLocationStruct,
    unit::Unit as UnitStruct,
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, CaseStatement, Expr, ExprTrait, Func, JoinType, Order, Query, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{
    entity::Entity, permission::Permission, storage::Storage, storelocation::StoreLocation,
    unit::Unit,
};

/// A stock line read from the database.
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
                product: Product::default(),
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

/// Returns the stock of a product for each store location the person can see,
/// one line per base unit, e.g. `1.5 L` for `1 L` and `500 mL`.
///
/// The base unit of a unit is its parent, or itself without parent. Archived
/// storages and history rows are not counted.
///
/// Sample request:
/// ```sql
/// SELECT store_location_id, store_location_full_path,
///   SUM(storage_quantity * self_unit.unit_multiplier) AS quantity,
///   parent_unit.unit_id, parent_unit.unit_label
/// FROM storage
/// JOIN store_location ON storage.store_location = store_location.store_location_id
/// LEFT JOIN unit self_unit ON storage.unit_quantity = self_unit.unit_id
/// LEFT JOIN unit parent_unit
///   ON parent_unit.unit_id = COALESCE(self_unit.unit, self_unit.unit_id)
/// WHERE storage.product = 6144 AND EXISTS (SELECT ... FROM permission ...)
/// GROUP BY store_location_id, parent_unit.unit_id
/// ```
pub fn compute_stock(
    db_connection: &Connection,
    product_id: u64,
    person_id: u64,
) -> Result<Vec<Stock>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("product_id:{product_id:?}");
    debug!("person_id:{person_id:?}");

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
        // The base unit: the parent unit, or the unit itself without parent.
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("parent_unit"),
            Expr::col((Alias::new("parent_unit"), Unit::UnitId)).eq(Func::coalesce([
                Expr::col((Alias::new("self_unit"), Unit::Unit)),
                Expr::col((Alias::new("self_unit"), Unit::UnitId)),
            ])),
        )
        .join(
            JoinType::InnerJoin,
            Entity::Table,
            Expr::col((StoreLocation::Table, StoreLocation::Entity))
                .equals((Entity::Table, Entity::EntityId)),
        )
        // A permission to read the storages of the entity. `EXISTS`, not a
        // join: a person with several matching permissions would count each
        // storage several times.
        .and_where(Expr::exists(
            Query::select()
                .expr(Expr::val(1))
                .from(Permission::Table)
                .and_where(Expr::col((Permission::Table, Permission::Person)).eq(person_id))
                .and_where(
                    Expr::col((Permission::Table, Permission::PermissionItem))
                        .is_in(["all", "storages"]),
                )
                .and_where(
                    Expr::col((Permission::Table, Permission::PermissionName))
                        .is_in(["r", "w", "all"]),
                )
                .and_where(
                    Expr::col((Permission::Table, Permission::PermissionEntity))
                        .equals((Entity::Table, Entity::EntityId))
                        .or(Expr::col((Permission::Table, Permission::PermissionEntity)).is_null()),
                )
                .take(),
        ))
        .and_where(Expr::col((Storage::Table, Storage::Product)).eq(product_id))
        .and_where(Expr::col((Storage::Table, Storage::StorageArchive)).eq(false))
        // Not the history rows: earlier versions of the storages.
        .and_where(Expr::col((Storage::Table, Storage::Storage)).is_null())
        .group_by_col((StoreLocation::Table, StoreLocation::StoreLocationId))
        .group_by_col((Alias::new("parent_unit"), Unit::UnitId))
        .order_by_expr(
            Expr::cust_with_expr(
                "? COLLATE NOCASE",
                Expr::col(StoreLocation::StoreLocationFullPath),
            ),
            Order::Asc,
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.as_str());
    debug!("select_values: {select_values:?}");

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

    debug!("stocks: {stocks:#?}");

    Ok(stocks)
}

#[cfg(test)]
#[path = "stock_tests.rs"]
mod stock_tests;
