use std::{
    fmt::{Display, Formatter},
    io::BufWriter,
    time::{SystemTime, UNIX_EPOCH},
};

use chimitheque_types::{
    borrowing::Borrowing as BorrowingStruct, entity::Entity as EntityStruct, error::ParseError,
    name::Name as NameStruct, person::Person as PersonStruct, product::Product as ProductStruct,
    requestfilter::RequestFilter, storage::Storage as StorageStruct,
    storelocation::StoreLocation as StoreLocationStruct, supplier::Supplier as SupplierStruct,
    unit::Unit as UnitStruct,
};
use chrono::{DateTime, Utc};
use csv::WriterBuilder;
use log::debug;
use qrcode_png::{Color, QrCode, QrCodeEcc};
use regex::Regex;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{
    any, Alias, ColumnRef, Cond, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SimpleExpr,
    SqliteQueryBuilder,
};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;

use crate::{
    bookmark::Bookmark,
    borrowing::Borrowing,
    casnumber::CasNumber,
    category::Category,
    cenumber::CeNumber,
    empiricalformula::EmpiricalFormula,
    entity::Entity,
    hazardstatement::HazardStatement,
    name::Name,
    permission::Permission,
    person::Person,
    precautionarystatement::PrecautionaryStatement,
    producer::Producer,
    producerref::ProducerRef,
    product::Product,
    producthazardstatements::Producthazardstatements,
    productprecautionarystatements::Productprecautionarystatements,
    productsymbols::Productsymbols,
    productsynonyms::Productsynonyms,
    producttags::Producttags,
    signalword::SignalWord,
    storelocation::{get_store_locations, StoreLocation},
    supplier::Supplier,
    symbol::Symbol,
    tag::Tag,
    unit::Unit,
};

#[derive(Debug, PartialEq, Eq)]
pub enum StorageError {
    MissingProductId,
    MissingPersonId,
    MissingStoreLocationId,
    MissingEntityId,
    MissingEntity,
    MissingUnitId,
    MissingStorageId,
    MissingSupplierId,

    StoreLocationNotFoundForId(u64),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            StorageError::MissingProductId => write!(f, "missing product id"),
            StorageError::MissingStoreLocationId => write!(f, "missing store location id"),
            StorageError::MissingEntityId => write!(f, "missing entity id"),
            StorageError::MissingEntity => write!(f, "missing entity"),
            StorageError::MissingPersonId => write!(f, "missing person id"),
            StorageError::MissingUnitId => write!(f, "missing unit id"),
            StorageError::MissingStorageId => write!(f, "missing storage id"),
            StorageError::MissingSupplierId => write!(f, "missing supplier id"),
            StorageError::StoreLocationNotFoundForId(id) => {
                write!(f, "store location not found for id {}", id)
            }
        }
    }
}

impl std::error::Error for StorageError {}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Storage {
    Table,
    StorageId,
    StorageCreationDate,
    StorageModificationDate,
    StorageEntryDate,
    StorageExitDate,
    StorageOpeningDate,
    StorageExpirationDate,
    StorageQuantity,
    StorageBarecode,
    StorageComment,
    StorageReference,
    StorageBatchNumber,
    StorageToDestroy,
    StorageArchive,
    StorageQrcode,
    StorageConcentration,
    // StorageNumberOfUnit,
    StorageNumberOfBag,
    StorageNumberOfCarton,
    Person,
    Product,
    StoreLocation,
    UnitConcentration,
    UnitQuantity,
    Supplier,
    Storage,
}

#[derive(Debug, Serialize)]
pub struct StorageWrapper(pub StorageStruct);

impl TryFrom<&Row<'_>> for StorageWrapper {
    type Error = ParseError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let maybe_unit_quantity: Option<u64> = row.get_unwrap("unit_quantity_unit_id");
        let maybe_unit_concentration: Option<u64> = row.get_unwrap("unit_concentration_unit_id");
        let maybe_parent_storage: Option<u64> = row.get_unwrap("parent_storage_id");
        let maybe_borrowing: Option<u64> = row.get_unwrap("borrowing_id");
        let maybe_supplier: Option<u64> = row.get_unwrap("supplier_id");

        let storage_creation_date_string: i64 = row.get_unwrap("storage_creation_date");
        let storage_creation_date = DateTime::from_timestamp(storage_creation_date_string, 0);

        let storage_modification_date_string: i64 = row.get_unwrap("storage_modification_date");
        let storage_modification_date =
            DateTime::from_timestamp(storage_modification_date_string, 0);

        let mut storage_entry_date: Option<DateTime<Utc>> = None;
        if let Some(storage_entry_date_string) =
            row.get_unwrap::<&str, Option<i64>>("storage_entry_date")
        {
            storage_entry_date = DateTime::from_timestamp(storage_entry_date_string, 0);
        }

        let mut storage_exit_date: Option<DateTime<Utc>> = None;
        if let Some(storage_exit_date_string) =
            row.get_unwrap::<&str, Option<i64>>("storage_exit_date")
        {
            storage_exit_date = DateTime::from_timestamp(storage_exit_date_string, 0);
        }

        let mut storage_opening_date: Option<DateTime<Utc>> = None;
        if let Some(storage_opening_date_string) =
            row.get_unwrap::<&str, Option<i64>>("storage_opening_date")
        {
            storage_opening_date = DateTime::from_timestamp(storage_opening_date_string, 0);
        }

        let mut storage_expiration_date: Option<DateTime<Utc>> = None;
        if let Some(storage_expiration_date_string) =
            row.get_unwrap::<&str, Option<i64>>("storage_expiration_date")
        {
            storage_expiration_date = DateTime::from_timestamp(storage_expiration_date_string, 0);
        }

        Ok(Self(StorageStruct {
            storage_id: row.get_unwrap("storage_id"),
            storage_creation_date: storage_creation_date.unwrap(),
            storage_modification_date: storage_modification_date.unwrap(),
            storage_entry_date,
            storage_exit_date,
            storage_opening_date,
            storage_expiration_date,
            storage_comment: row.get_unwrap("storage_comment"),
            storage_reference: row.get_unwrap("storage_reference"),
            storage_batch_number: row.get_unwrap("storage_batch_number"),
            storage_quantity: row.get_unwrap("storage_quantity"),
            storage_barecode: row.get_unwrap("storage_barecode"),
            storage_qrcode: row.get_unwrap("storage_qrcode"),
            storage_to_destroy: row.get_unwrap("storage_to_destroy"),
            storage_archive: row.get_unwrap("storage_archive"),
            storage_concentration: row.get_unwrap("storage_concentration"),
            storage_number_of_bag: row.get_unwrap("storage_number_of_bag"),
            storage_number_of_carton: row.get_unwrap("storage_number_of_carton"),

            person: PersonStruct {
                person_id: row.get_unwrap("person_id"),
                person_email: row.get_unwrap("person_email"),
                ..Default::default()
            },
            product: ProductStruct {
                product_id: row.get_unwrap("product_id"),
                name: NameStruct {
                    name_id: row.get_unwrap("name_id"),
                    name_label: row.get_unwrap("name_label"),
                    ..Default::default()
                },
                ..Default::default()
            },
            store_location: StoreLocationStruct {
                store_location_id: row.get_unwrap("store_location_id"),
                store_location_name: row.get_unwrap("store_location_name"),
                store_location_full_path: row.get_unwrap("store_location_full_path"),
                entity: Some(EntityStruct {
                    entity_id: row.get_unwrap("entity_id"),
                    entity_name: row.get_unwrap("entity_name"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            supplier: maybe_supplier.map(|_| SupplierStruct {
                supplier_id: row.get_unwrap("supplier_id"),
                supplier_label: row.get_unwrap("supplier_label"),
                ..Default::default()
            }),
            unit_quantity: maybe_unit_quantity.map(|_| UnitStruct {
                unit_id: row.get_unwrap("unit_quantity_unit_id"),
                unit_label: row.get_unwrap("unit_quantity_unit_label"),
                ..Default::default()
            }),
            unit_concentration: maybe_unit_concentration.map(|_| UnitStruct {
                unit_id: row.get_unwrap("unit_concentration_unit_id"),
                unit_label: row.get_unwrap("unit_concentration_unit_label"),
                ..Default::default()
            }),
            storage: maybe_parent_storage.map(|_| {
                Box::new(StorageStruct {
                    storage_id: row.get_unwrap("parent_storage_id"),
                    ..Default::default()
                })
            }),
            borrowing: maybe_borrowing.map(|_| BorrowingStruct {
                borrowing_id: row.get_unwrap("borrowing_id"),
                borrowing_comment: row.get_unwrap("borrowing_comment"),
                borrower: PersonStruct {
                    person_id: row.get_unwrap("borrower_person_id"),
                    person_email: row.get_unwrap("borrower_person_email"),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default() // storage_hc: row.get_unwrap("storage_hc"),
        }))
    }
}

fn populate_history_count(
    db_connection: &Connection,
    storage: &mut [StorageStruct],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for storage in storage.iter_mut() {
        let storage_id = storage.storage_id;

        // Create count query.
        let (count_sql, count_values) = Query::select()
            .expr(Expr::col((Storage::Table, Storage::StorageId)).count_distinct())
            .from(Storage::Table)
            .and_where(Expr::col((Storage::Table, Storage::Storage)).eq(storage_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("count_sql: {}", count_sql.clone().as_str());
        debug!("count_values: {:?}", count_values);

        // Perform count query.
        let mut stmt = db_connection.prepare(count_sql.as_str())?;
        let mut rows = stmt.query(&*count_values.as_params())?;
        let count: u64 = if let Some(row) = rows.next()? {
            row.get_unwrap(0)
        } else {
            0
        };

        storage.storage_hc = count;
    }

    Ok(())
}

pub fn export_storages(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    let (storages, _) = get_storages(db_connection, filter, person_id)?;

    let vec: Vec<u8> = vec![];
    let buffer = BufWriter::new(vec);

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(buffer);

    wtr.write_record(["BARECODE", "QUANTITY", "UNIT", "SUPPLIER", "STORE_LOCATION"])?;

    for storage in storages {
        wtr.serialize((
            storage.storage_barecode,
            storage.storage_quantity,
            storage.unit_quantity.unwrap_or_default().unit_label,
            storage.supplier.unwrap_or_default().supplier_label,
            storage.store_location.store_location_name,
        ))?;
    }
    wtr.flush()?;

    let inner_buffer_content = wtr.into_inner()?;

    let csv = String::from_utf8(inner_buffer_content.into_inner()?).unwrap();

    debug!("csv:{:?}", csv);

    Ok(csv)
}

pub fn get_storages(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<StorageStruct>, usize), Box<dyn std::error::Error + Send + Sync>> {
    debug!("filter:{:?}", filter);
    debug!("person_id:{:?}", person_id);

    // Does the person has the permission to access the restricted products?
    // We do not use the person::get_people function to retrieve the person as this function
    // retrieves a lot of information.
    let (exist_sql, exist_values) = Query::select()
        .expr(
            Expr::case(
                Expr::exists(
                    Query::select()
                        .expr(Expr::col((Permission::Table, Permission::PermissionItem)))
                        .from(Permission::Table)
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionItem))
                                .is_in(["rproducts", "all"]),
                        )
                        .and_where(Expr::col((Permission::Table, Permission::Person)).eq(person_id))
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionName)).ne("n"),
                        )
                        .take(),
                ),
                Expr::val(true),
            )
            .finally(Expr::val(false)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("exist_sql: {}", exist_sql.clone().as_str());
    debug!("exist_values: {:?}", exist_values);

    // Perform exist query.
    let mut stmt = db_connection.prepare(exist_sql.as_str())?;
    let mut rows = stmt.query(&*exist_values.as_params())?;
    let has_rproducts_permission: bool = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        false
    };

    let order_by: ColumnRef = if let Some(order_by_string) = filter.order_by {
        match order_by_string.as_str() {
            "product.name.name_label" => (Name::Table, Name::NameLabel).into_column_ref(),
            "storage_batch_number" => {
                (Storage::Table, Storage::StorageBatchNumber).into_column_ref()
            }
            "store_location.store_location_full_path" => {
                (StoreLocation::Table, StoreLocation::StoreLocationFullPath).into_column_ref()
            }
            "storage_modification_date" => {
                (Storage::Table, Storage::StorageModificationDate).into_column_ref()
            }
            _ => (Storage::Table, Storage::StorageId).into_column_ref(),
        }
    } else {
        (Storage::Table, Storage::StorageId).into_column_ref()
    };

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Storage::Table)
        //
        // product
        //
        .join(
            JoinType::LeftJoin,
            Product::Table,
            Expr::col((Storage::Table, Storage::Product)).equals((Product::Table, Product::ProductId)),
        )
        //
        // parent storage
        //
        .join_as(
            JoinType::LeftJoin,
            Storage::Table,
            Alias::new("parent"),
            Expr::col((Storage::Table, Storage::Storage))
                .equals((Alias::new("parent"), Alias::new("storage_id"))),
        )
        //
        // name
        //
        .join(
            JoinType::LeftJoin,
            Name::Table,
            Expr::col((Product::Table, Product::Name)).equals((Name::Table, Name::NameId)),
        )
        //
        // person
        //
        .join(
            JoinType::LeftJoin,
            Person::Table,
            Expr::col((Storage::Table, Storage::Person)).equals((Person::Table, Person::PersonId)),
        )
        //
        // cas number
        //
        .join(
            JoinType::LeftJoin,
            CasNumber::Table,
            Expr::col((Product::Table, Product::CasNumber))
                .equals((CasNumber::Table, CasNumber::CasNumberId)),
        )
        //
        // ce number
        //
        .join(
            JoinType::LeftJoin,
            CeNumber::Table,
            Expr::col((Product::Table, Product::CeNumber))
                .equals((CeNumber::Table, CeNumber::CeNumberId)),
        )
        //
        // empirical formula
        //
        .join(
            JoinType::LeftJoin,
            EmpiricalFormula::Table,
            Expr::col((Product::Table, Product::EmpiricalFormula)).equals((
                EmpiricalFormula::Table,
                EmpiricalFormula::EmpiricalFormulaId,
            )),
        )
        //
        // signal word
        //
        .join(
            JoinType::LeftJoin,
            SignalWord::Table,
            Expr::col((Product::Table, Product::SignalWord))
                .equals((SignalWord::Table, SignalWord::SignalWordId)),
        )
        //
        // category
        //
        .join(
            JoinType::LeftJoin,
            Category::Table,
            Expr::col((Product::Table, Product::Category))
                .equals((Category::Table, Category::CategoryId)),
        )
        //
        // producerref
        //
        .join(
            JoinType::LeftJoin,
            ProducerRef::Table,
            Expr::col((Product::Table, Product::ProducerRef))
                .equals((ProducerRef::Table, ProducerRef::ProducerRefId)),
        )
        .join(
            JoinType::LeftJoin,
            Producer::Table,
            Expr::col((ProducerRef::Table, ProducerRef::Producer))
                .equals((Producer::Table, Producer::ProducerId)),
        )
        //
        // hazard statements
        //
        .join(
            JoinType::LeftJoin,
            Producthazardstatements::Table,
            Expr::col((
                Producthazardstatements::Table,
                Producthazardstatements::ProducthazardstatementsProductId,
            ))
            .equals((Product::Table, Product::ProductId)),
        )
        .join(
            JoinType::LeftJoin,
            HazardStatement::Table,
            Expr::col((HazardStatement::Table, HazardStatement::HazardStatementId)).equals((
                Producthazardstatements::Table,
                Producthazardstatements::ProducthazardstatementsHazardStatementId,
            )),
        )
        //
        // precautionary statements
        //
        .join(
            JoinType::LeftJoin,
            Productprecautionarystatements::Table,
            Expr::col((
                Productprecautionarystatements::Table,
                Productprecautionarystatements::ProductprecautionarystatementsProductId,
            ))
            .equals((Product::Table, Product::ProductId)),
        )
        .join(
            JoinType::LeftJoin,
            PrecautionaryStatement::Table,
            Expr::col((PrecautionaryStatement::Table, PrecautionaryStatement::PrecautionaryStatementId)).equals((
                Productprecautionarystatements::Table,
                Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId,
            )),
        )
        //
        // symbols
        //
        .join(
            JoinType::LeftJoin,
            Productsymbols::Table,
            Expr::col((
                Productsymbols::Table,
                Productsymbols::ProductsymbolsProductId,
            ))
            .equals((Product::Table, Product::ProductId)),
        )
        .join(
            JoinType::LeftJoin,
            Symbol::Table,
            Expr::col((Symbol::Table, Symbol::SymbolId)).equals((
                Productsymbols::Table,
                Productsymbols::ProductsymbolsSymbolId,
            )),
        )
        //
        // tags
        //
        .join(
            JoinType::LeftJoin,
            Producttags::Table,
            Expr::col((
                Producttags::Table,
                Producttags::ProducttagsProductId,
            ))
            .equals((Product::Table, Product::ProductId)),
        )
        .join(
            JoinType::LeftJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::TagId)).equals((
                Producttags::Table,
                Producttags::ProducttagsTagId,
            )),
        )
        //
        // supplier
        //
        .join(
            JoinType::LeftJoin,
            Supplier::Table,
            Expr::col((Storage::Table, Storage::Supplier))
                .equals((
                    Supplier::Table,
                    Supplier::SupplierId,
                )),
        )
        //
        // unit quantity
        //
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("unit_quantity"),
            Expr::col((Storage::Table, Storage::UnitQuantity))
                .equals((Alias::new("unit_quantity"), Unit::UnitId)),
        )
        //
        // unit concentration
        //
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("unit_concentration"),
            Expr::col((Storage::Table, Storage::UnitConcentration))
                .equals((Alias::new("unit_concentration"), Unit::UnitId)),
        )
        //
        // bookmarks
        //
        .join(
            JoinType::LeftJoin,
            Bookmark::Table,
            Expr::col((Bookmark::Table, Bookmark::Product)).equals((
                Product::Table,
                Product::ProductId,
            )).and(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id)
        ))
        //
        // borrowing, borrower
        //
        .join(
            JoinType::LeftJoin,
            Borrowing::Table,
            Expr::col((Borrowing::Table, Borrowing::Storage)).equals((
                Storage::Table,
                Storage::StorageId,
            )).and(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id)),
        )
        .join_as(
            JoinType::LeftJoin,
            Person::Table,
            Alias::new("borrower"),
            Expr::col((Alias::new("borrower"), Person::PersonId))
        .equals((Borrowing::Table, Borrowing::Borrower)))
        //
        .join(
            // storelocation
            JoinType::LeftJoin,
            StoreLocation::Table,
            Expr::col((Storage::Table, Storage::StoreLocation))
            .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
        )
        .join(
            // entity
            JoinType::LeftJoin,
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
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity"))).eq(-1),
                        ),
                ),
        )
        //
        // restricted products?
        //
        .conditions(
            !has_rproducts_permission,
            |q| {
            q.and_where(
                Expr::col((Product::Table, Product::ProductRestricted))
                .eq(false),
            );
        },
        |_| {},
        )
        .conditions(
            filter.show_chem,
            |q| {
            q.and_where(
                Expr::col((Product::Table, Product::ProductType))
                .eq("chem"),
            );
        },
        |_| {},
        )
        .conditions(
            filter.show_bio,
            |q| {
                q.and_where(
                    Expr::col((Product::Table, Product::ProductType))
                    .eq("bio"),
                );
            },
            |_| {},
        )
        .conditions(
            filter.show_consu,
            |q| {
                q.and_where(
                    Expr::col((Product::Table, Product::ProductType))
                    .eq("cons"),
                );
            },
            |_| {},
        )
        .conditions(
            !filter.history,
            |q| {
            q.and_where(
                Expr::col((Storage::Table, Storage::Storage))
                .is_null(),
            );
        },
        |_| {},
        )
        .conditions(
            filter.id.is_some() && filter.history,
                    |q| {
                        q.and_where(
                            Expr::col((Storage::Table, Storage::StorageId))
                            .eq(filter.id.unwrap()).or(
                                Expr::col((Storage::Table, Storage::Storage))
                                .eq(filter.id.unwrap())
                            ),
                        );
                    },
                    |_| {},
        )
        .conditions(
            // getting storages with identical barecode
            filter.id.is_some() && !filter.history,
                    |q| {
                        q.and_where(
                            Expr::col((Storage::Table, Storage::StorageId))
                            .eq(filter.id.unwrap()).or(
                                Expr::col((Storage::Table, Storage::StorageBarecode))
                                .in_subquery(
                                    Query::select()
                                        .from(Storage::Table)
                                        .expr(Expr::col((Storage::Table, Storage::StorageBarecode)))
                                        .and_where(Expr::col((Storage::Table, Storage::StorageId))
                                        .eq(filter.id.unwrap())).take())
                            ),
                        );
                    },
                    |_| {},
        )
        .conditions(
            filter.product.is_some(),
                    |q| {
                        q.and_where(
                            Expr::col((Storage::Table, Storage::Product))
                            .eq(filter.product.unwrap()),
                        );
                    },
                    |_| {},
        )
        .conditions(
            filter.custom_name_part_of.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Name::Table, Name::NameLabel))
                        .like(format!("%{}%", filter.custom_name_part_of.clone().unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.cas_number.is_some(),
            |q| {
                q.and_where(Expr::col(Product::CasNumber).eq(filter.cas_number.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.cas_number_string.is_some(),
                    |q| {
                        q.and_where(Expr::col(CasNumber::CasNumberLabel).eq(filter.cas_number_string.unwrap()));
                    },
                    |_| {},
        )
        .conditions(
            filter.empirical_formula.is_some(),
                    |q| {
                        q.and_where(Expr::col(Product::EmpiricalFormula).eq(filter.empirical_formula.unwrap()));
                    },
                    |_| {},
        )
        .conditions(
            filter.is_cmr,
            |q| {
                q.and_where(Expr::col(CasNumber::CasNumberCmr).is_not_null())
                    .and_where(Expr::col(HazardStatement::HazardStatementCmr).is_not_null());
            },
            |_| {},
        )
        .conditions(
            filter.name.is_some(),
            |q| {
                q.join(
                    // synonyms
                    JoinType::LeftJoin,
                    Productsynonyms::Table,
                    Expr::col((Productsynonyms::Table, Productsynonyms::ProductsynonymsProductId))
                    .equals((Product::Table, Product::ProductId)),
                );
                q.cond_where(
                  any![
                      Expr::col(Product::Name).eq(filter.name.unwrap()),
                      Expr::col(Productsynonyms::ProductsynonymsNameId).eq(filter.name.unwrap()),
                  ]
                );
            },
            |_| {},
        )
        .conditions(
            filter.store_location.is_some(),
            |q| {
                q.and_where(
                    Expr::col(StoreLocation::StoreLocationId).eq(filter.store_location.unwrap()),
                );
            },
            |_| {},
        )
        .conditions(
            filter.entity.is_some(),
                    |q| {
                        q.and_where(
                            Expr::col(Entity::EntityId).eq(filter.entity.unwrap()),
                        );
                    },
                    |_| {},
        )
        .conditions(
            filter.storage_to_destroy,
            |q| {
                q.and_where(Expr::col((Storage::Table,Storage::StorageToDestroy)).eq(filter.storage_to_destroy));
            },
            |_| {},
        )
        .conditions(
            filter.storage_barecode.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Storage::Table,Storage::StorageBarecode)).like(format!("%{}%",filter.storage_barecode.unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.storage_batch_number.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Storage::Table,Storage::StorageBatchNumber)).like(format!("%{}%",filter.storage_batch_number.unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.storage_archive.is_some_and(|x| x),
            |q| {
                q.and_where(
                    Expr::col((Storage::Table,Storage::StorageArchive)).eq(true),
                );
            },
            |_| {},
        )
        .conditions(
            filter.storage_archive.is_some_and(|x| !x),
            |q| {
                q.and_where(
                    Expr::col((Storage::Table,Storage::StorageArchive)).eq(false),
                );
            },
            |_| {},
        )
        .conditions(
            filter.borrowing,
            |q| {
                q.join(
                    // borrowing
                    JoinType::Join,
                    Borrowing::Table,
                    Expr::col((Borrowing::Table, Borrowing::Storage))
                        .equals((Storage::Table, Storage::StorageId)),
                );
                q.and_where(Expr::col((Borrowing::Table, Borrowing::Borrower)).eq(person_id));
            },
            |_| {},)
        .conditions(
            filter.producer_ref.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Product::ProducerRef).eq(filter.producer_ref.unwrap()),
                );
            },
            |_| {},)
        .conditions(
            filter.bookmark,
            |q| {
                q.join(
                    // bookmark
                    JoinType::Join,
                    Bookmark::Table,
                    Expr::col((Bookmark::Table, Bookmark::Product))
                    .equals((Product::Table, Product::ProductId)),
                );
                q.and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id));
            },
            |_| {},)
        .conditions(
            filter.tags.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Producttags::ProducttagsTagId).is_in(filter.tags.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.symbols.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Productsymbols::ProductsymbolsSymbolId).is_in(filter.symbols.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.hazard_statements.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Producthazardstatements::ProducthazardstatementsHazardStatementId).is_in(filter.hazard_statements.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.precautionary_statements.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId).is_in(filter.precautionary_statements.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.category.is_some(),
            |q| {
                q.and_where(Expr::col(Product::Category).eq(filter.category.unwrap()));
            },
            |_| {},
        )
        .conditions(
            filter.signal_word.is_some(),
            |q| {
                q.and_where(Expr::col(Product::SignalWord).eq(filter.signal_word.unwrap()));
            },
            |_| {},
            );

    // Create count query.
    let (count_sql, count_values) = expression
        .clone()
        .expr(Expr::col((Storage::Table, Storage::StorageId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .expr(Expr::col((Storage::Table, Storage::StorageId)))
        .expr(Expr::col((Storage::Table, Storage::StorageCreationDate)))
        .expr(Expr::col((
            Storage::Table,
            Storage::StorageModificationDate,
        )))
        .expr(Expr::col((Storage::Table, Storage::StorageEntryDate)))
        .expr(Expr::col((Storage::Table, Storage::StorageExitDate)))
        .expr(Expr::col((Storage::Table, Storage::StorageOpeningDate)))
        .expr(Expr::col((Storage::Table, Storage::StorageExpirationDate)))
        .expr(Expr::col((Storage::Table, Storage::StorageComment)))
        .expr(Expr::col((Storage::Table, Storage::StorageReference)))
        .expr(Expr::col((Storage::Table, Storage::StorageBatchNumber)))
        .expr(Expr::col((Storage::Table, Storage::StorageQuantity)))
        .expr(Expr::col((Storage::Table, Storage::StorageBarecode)))
        .expr(Expr::col((Storage::Table, Storage::StorageQrcode)))
        .expr(Expr::col((Storage::Table, Storage::StorageToDestroy)))
        .expr(Expr::col((Storage::Table, Storage::StorageArchive)))
        .expr(Expr::col((Storage::Table, Storage::StorageConcentration)))
        .expr(Expr::col((Storage::Table, Storage::StorageNumberOfBag)))
        .expr(Expr::col((Storage::Table, Storage::StorageNumberOfCarton)))
        .expr(Expr::col((Person::Table, Person::PersonId)))
        .expr(Expr::col((Person::Table, Person::PersonEmail)))
        .expr(Expr::col((Product::Table, Product::ProductId)))
        .expr(Expr::col((Name::Table, Name::NameId)))
        .expr(Expr::col((Name::Table, Name::NameLabel)))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationId,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationName,
        )))
        .expr(Expr::col((
            StoreLocation::Table,
            StoreLocation::StoreLocationFullPath,
        )))
        .expr(Expr::col((Entity::Table, Entity::EntityId)))
        .expr(Expr::col((Entity::Table, Entity::EntityName)))
        .expr(Expr::col((Supplier::Table, Supplier::SupplierId)))
        .expr(Expr::col((Supplier::Table, Supplier::SupplierLabel)))
        .expr_as(
            Expr::col((Alias::new("unit_quantity"), Alias::new("unit_id"))),
            Alias::new("unit_quantity_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_quantity"), Alias::new("unit_label"))),
            Alias::new("unit_quantity_unit_label"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_concentration"), Alias::new("unit_id"))),
            Alias::new("unit_concentration_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_concentration"), Alias::new("unit_label"))),
            Alias::new("unit_concentration_unit_label"),
        )
        .expr_as(
            Expr::col((Alias::new("parent"), Alias::new("storage_id"))),
            Alias::new("parent_storage_id"),
        )
        .expr_as(
            Expr::col((Alias::new("borrower"), Alias::new("person_id"))),
            Alias::new("borrower_person_id"),
        )
        .expr_as(
            Expr::col((Alias::new("borrower"), Alias::new("person_email"))),
            Alias::new("borrower_person_email"),
        )
        .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingId)))
        .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingComment)))
        .order_by(order_by, order)
        .group_by_col((Storage::Table, Storage::StorageId))
        .conditions(
            filter.limit.is_some(),
            |q| {
                q.limit(filter.limit.unwrap());
            },
            |_| {},
        )
        .conditions(
            filter.offset.is_some(),
            |q| {
                q.offset(filter.offset.unwrap());
            },
            |_| {},
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("select_sql: {}", select_sql.clone().as_str());
    debug!("select_values: {:?}", select_values);

    // Perform count query.
    let mut stmt = db_connection.prepare(count_sql.as_str())?;
    let mut rows = stmt.query(&*count_values.as_params())?;
    let count: usize = if let Some(row) = rows.next()? {
        row.get_unwrap(0)
    } else {
        0
    };

    // Perform select query.
    let mut stmt = db_connection.prepare(select_sql.as_str())?;
    let mut storages = Vec::new();
    let mut rows = stmt.query(&*select_values.as_params())?;
    while let Some(row) = rows.next()? {
        let storage = StorageWrapper::try_from(row)?;
        storages.push(storage.0);
    }

    populate_history_count(db_connection, &mut storages)?;

    debug!("storages: {:#?}", storages);

    Ok((storages, count))
}

fn create_storage_qrcode(
    db_transaction: &Transaction,
    storage_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut qrcode = QrCode::new(storage_id.to_string(), QrCodeEcc::Medium).unwrap();

    qrcode.margin(10);
    qrcode.zoom(10);

    let buffer = qrcode.generate(Color::Grayscale(0, 255)).unwrap();

    let (update_sql, update_values) = Query::update()
        .table(Storage::Table)
        .values([(Storage::StorageQrcode, buffer.into())])
        .and_where(Expr::col(Storage::StorageId).eq(storage_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("update_sql: {}", update_sql.clone().as_str());
    debug!("update_values: {:?}", update_values);

    _ = db_transaction.execute(update_sql.as_str(), &*update_values.as_params())?;

    Ok(())
}

fn create_storage_history(
    db_transaction: &Transaction,
    storage: &StorageStruct,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    db_transaction.execute(
        &format!(
            "INSERT into storage (storage_creation_date,
		storage_modification_date,
		storage_entry_date,
		storage_exit_date,
		storage_opening_date,
		storage_expiration_date,
		storage_comment,
		storage_reference,
		storage_batch_number,
		storage_quantity,
		storage_barecode,
		storage_to_destroy,
		storage_archive,
		storage_concentration,
		storage_number_of_bag,
		storage_number_of_carton,
		person,
		product,
		store_location,
		unit_quantity,
		unit_concentration,
		supplier,
		storage) SELECT storage_creation_date,
				storage_modification_date,
				storage_entry_date,
				storage_exit_date,
				storage_opening_date,
				storage_expiration_date,
				storage_comment,
				storage_reference,
				storage_batch_number,
				storage_quantity,
				storage_barecode,
				storage_to_destroy,
				storage_archive,
				storage_concentration,
				storage_number_of_bag,
				storage_number_of_carton,
				person,
				product,
				store_location,
				unit_quantity,
				unit_concentration,
				supplier,
				{} FROM storage WHERE storage_id = (?1)",
            storage.storage_id.unwrap(),
        ),
        [storage.storage_id.unwrap()],
    )?;

    Ok(())
}

fn compute_storage_barecode_parts(
    db_transaction: &Transaction,
    storage: &StorageStruct,
    product_id: u64,
    person_id: u64,
) -> Result<(String, u64, u64), Box<dyn std::error::Error + Send + Sync>> {
    let store_location_id = match storage.store_location.store_location_id {
        Some(store_location_id) => store_location_id,
        None => return Err(Box::new(StorageError::MissingStoreLocationId)),
    };

    // Getting the store location and its name..
    let (store_locations, nb_results) = get_store_locations(
        db_transaction,
        RequestFilter {
            id: Some(store_location_id),
            ..Default::default()
        },
        person_id,
    )?;

    if nb_results == 0 {
        return Err(format!("no store location found for id {}", store_location_id).into());
    };

    let store_location = store_locations.first().unwrap();
    let store_location_name = store_location.clone().store_location_name;

    // Getting the entity id.
    let entity_id = match store_location.entity.clone() {
        Some(entity) => match entity.entity_id {
            Some(entity_id) => entity_id,
            None => return Err(Box::new(StorageError::MissingEntityId)),
        },
        None => return Err(Box::new(StorageError::MissingEntity)),
    };

    debug!("store_location_id: {}", store_location_id);
    debug!("store_location_name: {}", store_location_name);
    debug!("product_id: {}", product_id);
    debug!("entity_id: {}", entity_id);

    let re = Regex::new(r#"^\[(?P<groupone>[_a-zA-Z]+)\].*$"#)?;
    let barecode_string = match re.captures(&store_location_name) {
        Some(caps) => caps["groupone"].to_string(),
        None => "_".to_string(),
    };

    debug!("barecode_string: {:#?}", barecode_string);

    let (maybe_barecode_major, maybe_barecode_minor): (Option<u64>, Option<u64>) = db_transaction
        .query_row(
            r#"
            SELECT CAST(regex_capture(
              "[_a-zA-Z]+(?P<barecode_major>[0-9]+)\.[0-9]+",
              storage_barecode,
              "barecode_major"
            ) AS INTEGER) as barecode_major,
            MAX(CAST(substr(storage_barecode, instr(storage_barecode, '.')+1) AS INTEGER)) as barecode_minor
            FROM storage
            JOIN store_location on storage.store_location = store_location.store_location_id
            WHERE product = (?1)
            AND store_location.entity = (?2)
            AND storage.storage IS NULL
            AND regexp('^[_a-zA-Z]+[0-9]+\.[0-9]+$', '' || storage_barecode || '') = true
		"#,
            [product_id, entity_id],
            |row| {
                        Ok((
                            row.get("barecode_major")?,
                            row.get("barecode_minor")?,
                        ))
                    },
        )?;

    debug!("maybe_barecode_major: {:#?}", maybe_barecode_major);
    debug!("barecode_minor: {:#?}", maybe_barecode_minor);

    let barecode_major = maybe_barecode_major.unwrap_or(product_id);
    let barecode_minor = maybe_barecode_minor.unwrap_or(1);

    Ok((barecode_string, barecode_major, barecode_minor))
}

pub fn create_update_storage(
    db_connection: &mut Connection,
    mut storage: StorageStruct,
    nb_items: u64,
    identical_barecode: bool,
) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("create_update_storage: {:#?}", storage);

    // Created storage ids
    let mut storage_ids = vec![];

    let product_id = match storage.product.product_id {
        Some(product_id) => product_id,
        None => return Err(Box::new(StorageError::MissingProductId)),
    };

    let person_id = match storage.person.person_id {
        Some(person_id) => person_id,
        None => return Err(Box::new(StorageError::MissingPersonId)),
    };

    let store_location_id = match storage.store_location.store_location_id {
        Some(store_location_id) => store_location_id,
        None => return Err(Box::new(StorageError::MissingStoreLocationId)),
    };

    let db_transaction = db_connection.transaction()?;

    //
    // Create history on update.
    //
    if storage.storage_id.is_some() {
        create_storage_history(&db_transaction, &storage)?;
    }

    //
    // Generate barcode if empty.
    //
    let mut barecode_string = "".to_string();
    let mut barecode_major: u64 = product_id;
    let mut barecode_minor: u64 = 1;
    if storage.storage_barecode.is_none() {
        (barecode_string, barecode_major, barecode_minor) =
            compute_storage_barecode_parts(&db_transaction, &storage, product_id, person_id)?;

        storage.storage_barecode = Some(format!(
            "{}{}.{}",
            barecode_string,
            barecode_major,
            barecode_minor + 1
        ));
    }

    // Create nb_items storages.
    let mut nb_items_created = 0;
    while nb_items_created < nb_items {
        let mut columns = vec![
            Storage::Product,
            Storage::Person,
            Storage::StoreLocation,
            Storage::StorageQrcode,
            Storage::StorageToDestroy,
            Storage::StorageArchive,
        ];
        let mut values = vec![
            SimpleExpr::Value(product_id.into()),
            SimpleExpr::Value(person_id.into()),
            SimpleExpr::Value(store_location_id.into()),
            SimpleExpr::Value(storage.storage_qrcode.clone().into()),
            SimpleExpr::Value(storage.storage_to_destroy.into()),
            SimpleExpr::Value(storage.storage_archive.into()),
        ];

        if let Some(storage_entry_date) = &storage.storage_entry_date {
            columns.push(Storage::StorageEntryDate);
            values.push(SimpleExpr::Value(storage_entry_date.timestamp().into()));
        }

        if let Some(storage_exit_date) = &storage.storage_exit_date {
            columns.push(Storage::StorageExitDate);
            values.push(SimpleExpr::Value(storage_exit_date.timestamp().into()));
        }

        if let Some(storage_opening_date) = &storage.storage_opening_date {
            columns.push(Storage::StorageOpeningDate);
            values.push(SimpleExpr::Value(storage_opening_date.timestamp().into()));
        }

        if let Some(storage_expiration_date) = &storage.storage_expiration_date {
            columns.push(Storage::StorageExpirationDate);
            values.push(SimpleExpr::Value(
                storage_expiration_date.timestamp().into(),
            ));
        }

        if let Some(storage_comment) = &storage.storage_comment {
            columns.push(Storage::StorageComment);
            values.push(SimpleExpr::Value(storage_comment.into()));
        }

        if let Some(storage_reference) = &storage.storage_reference {
            columns.push(Storage::StorageReference);
            values.push(SimpleExpr::Value(storage_reference.into()));
        }

        if let Some(storage_batch_number) = &storage.storage_batch_number {
            columns.push(Storage::StorageBatchNumber);
            values.push(SimpleExpr::Value(storage_batch_number.into()));
        }

        if let Some(storage_quantity) = storage.storage_quantity {
            columns.push(Storage::StorageQuantity);
            values.push(SimpleExpr::Value(storage_quantity.into()));
        }

        if let Some(storage_barecode) = &storage.storage_barecode {
            columns.push(Storage::StorageBarecode);
            values.push(SimpleExpr::Value(storage_barecode.into()));
        }

        if let Some(storage_concentration) = storage.storage_concentration {
            columns.push(Storage::StorageConcentration);
            values.push(SimpleExpr::Value(storage_concentration.into()));
        }

        if let Some(storage_number_of_bag) = storage.storage_number_of_bag {
            columns.push(Storage::StorageNumberOfBag);
            values.push(SimpleExpr::Value(storage_number_of_bag.into()));
        }

        if let Some(storage_number_of_carton) = storage.storage_number_of_carton {
            columns.push(Storage::StorageNumberOfCarton);
            values.push(SimpleExpr::Value(storage_number_of_carton.into()));
        }

        if let Some(supplier) = &storage.supplier {
            let supplier_id = match supplier.supplier_id {
                Some(supplier_id) => supplier_id,
                None => return Err(Box::new(StorageError::MissingSupplierId)),
            };

            columns.push(Storage::Supplier);
            values.push(SimpleExpr::Value(supplier_id.into()));
        }

        if let Some(unit_quantity) = storage.unit_quantity.clone() {
            let unit_id = match unit_quantity.unit_id {
                Some(unit_id) => unit_id,
                None => return Err(Box::new(StorageError::MissingUnitId)),
            };

            columns.push(Storage::UnitQuantity);
            values.push(SimpleExpr::Value(unit_id.into()));
        }

        if let Some(unit_concentration) = storage.unit_concentration.clone() {
            let unit_id = match unit_concentration.unit_id {
                Some(unit_id) => unit_id,
                None => return Err(Box::new(StorageError::MissingUnitId)),
            };

            columns.push(Storage::UnitConcentration);
            values.push(SimpleExpr::Value(unit_id.into()));
        }

        if let Some(storage) = storage.storage.clone() {
            let storage_id = match storage.storage_id {
                Some(storage_id) => storage_id,
                None => return Err(Box::new(StorageError::MissingStorageId)),
            };

            columns.push(Storage::Storage);
            values.push(SimpleExpr::Value(storage_id.into()));
        }

        let sql_query: String;
        let sql_values: RusqliteValues = RusqliteValues(vec![]);

        if let Some(storage_id) = storage.storage_id {
            // Update query - nb_items is supposed to be == 1.
            columns.push(Storage::StorageId);
            values.push(SimpleExpr::Value(storage_id.into()));

            columns.push(Storage::StorageModificationDate);
            values.push(SimpleExpr::Value(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .into(),
            ));

            sql_query = Query::insert()
                .replace()
                .into_table(Storage::Table)
                .columns(columns)
                .values(values)?
                .to_string(SqliteQueryBuilder);
        } else {
            // Insert query.
            // Storage creation date is set by database default.
            sql_query = Query::insert()
                .into_table(Storage::Table)
                .columns(columns)
                .values(values)?
                .to_string(SqliteQueryBuilder);
        }

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

        let last_insert_update_id: u64;

        if let Some(storage_id) = storage.storage_id {
            last_insert_update_id = storage_id;
        } else {
            last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;

            create_storage_qrcode(&db_transaction, last_insert_update_id)?;
        }

        debug!("last_insert_update_id: {}", last_insert_update_id);

        storage_ids.push(last_insert_update_id);

        create_storage_qrcode(&db_transaction, last_insert_update_id)?;

        if !identical_barecode {
            barecode_minor += 1;

            storage.storage_barecode = Some(format!(
                "{}{}.{}",
                barecode_string, barecode_major, barecode_minor
            ));
        }

        nb_items_created += 1;
    }

    db_transaction.commit()?;

    Ok(storage_ids)
}

pub fn delete_storage(
    db_connection: &mut Connection,
    storage_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("delete_storage: {:#?}", storage_id);

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Storage::Table)
        .cond_where(
            Cond::any()
                .add(Expr::col(Storage::StorageId).eq(storage_id))
                .add(Expr::col(Storage::Storage).eq(storage_id)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

pub fn archive_storage(
    db_connection: &mut Connection,
    storage_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("archive_storage: {:#?}", storage_id);

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![(Storage::StorageArchive, true.into())];

    let (sql_query, sql_values) = Query::update()
        .table(Storage::Table)
        .values(columns_values)
        .cond_where(
            Cond::any()
                .add(Expr::col(Storage::StorageId).eq(storage_id))
                .add(Expr::col(Storage::Storage).eq(storage_id)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}

pub fn unarchive_storage(
    db_connection: &mut Connection,
    storage_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("unarchive_storage: {:#?}", storage_id);

    // Update request: list of (columns, values) pairs to insert.
    let columns_values = vec![(Storage::StorageArchive, false.into())];

    let (sql_query, sql_values) = Query::update()
        .table(Storage::Table)
        .values(columns_values)
        .cond_where(
            Cond::any()
                .add(Expr::col(Storage::StorageId).eq(storage_id))
                .add(Expr::col(Storage::Storage).eq(storage_id)),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_connection.execute(&sql_query, &*sql_values.as_params())?;

    Ok(())
}
