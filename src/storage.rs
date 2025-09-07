use chimitheque_types::{
    borrowing::Borrowing as BorrowingStruct, entity::Entity as EntityStruct, error::ParseError,
    name::Name as NameStruct, person::Person as PersonStruct, product::Product as ProductStruct,
    requestfilter::RequestFilter, storage::Storage as StorageStruct,
    storelocation::StoreLocation as StoreLocationStruct, supplier::Supplier as SupplierStruct,
    unit::Unit as UnitStruct,
};
use chrono::{DateTime, Utc};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    any, Alias, ColumnRef, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;

use crate::{
    bookmark::Bookmark, borrowing::Borrowing, casnumber::CasNumber, category::Category,
    cenumber::CeNumber, empiricalformula::EmpiricalFormula, entity::Entity,
    hazardstatement::HazardStatement, name::Name, permission::Permission, person::Person,
    precautionarystatement::PrecautionaryStatement, producer::Producer, producerref::ProducerRef,
    product::Product, producthazardstatements::Producthazardstatements,
    productprecautionarystatements::Productprecautionarystatements, productsymbols::Productsymbols,
    productsynonyms::Productsynonyms, producttags::Producttags, signalword::SignalWord,
    storelocation::StoreLocation, supplier::Supplier, symbol::Symbol, tag::Tag, unit::Unit,
};

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
    StorageNumberOfUnit,
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
        dbg!(row);

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
                entities: None,
                managed_entities: None,
                permissions: None,
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
            entity: EntityStruct {
                entity_id: row.get_unwrap("entity_id"),
                entity_name: row.get_unwrap("entity_name"),
                ..Default::default()
            },
            store_location: StoreLocationStruct {
                store_location_id: row.get_unwrap("store_location_id"),
                store_location_name: row.get_unwrap("store_location_name"),
                store_location_full_path: row.get_unwrap("store_location_full_path"),
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
                ..Default::default()
            }),
            ..Default::default() // storage_nb_items: row.get_unwrap("storage_nb_items"),
                                 // storage_identical_barecode: row.get_unwrap("storage_identical_barecode"),
                                 // storage_hc: row.get_unwrap("storage_hc"),
        }))
    }
}

pub fn get_storages(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<StorageStruct>, usize), Box<dyn std::error::Error>> {
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
                        .expr(Expr::col((Permission::Table, Permission::PermissionId)))
                        .from(Permission::Table)
                        .and_where(
                            Expr::col((Permission::Table, Permission::PermissionItem))
                                .eq("rproducts"),
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
        // borrowing
        //
        .join(
            JoinType::LeftJoin,
            Borrowing::Table,
            Expr::col((Borrowing::Table, Borrowing::Storage)).equals((
                Storage::Table,
                Storage::StorageId,
            )).and(Expr::col((Borrowing::Table, Borrowing::Person)).eq(person_id)),
        )
        //
        // storage -> permissions
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
            filter.id.is_some(),
                    |q| {
                        q.and_where(
                            Expr::col((Storage::Table, Storage::StorageId))
                            .eq(filter.id.unwrap()),
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
                q.and_where(Expr::col(Storage::StorageToDestroy).eq(filter.storage_to_destroy));
            },
            |_| {},
        )
        .conditions(
            filter.storage_barecode.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Storage::StorageBarecode).like(format!("%{}%",filter.storage_barecode.unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.storage_batch_number.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Storage::StorageBatchNumber).like(format!("%{}%",filter.storage_batch_number.unwrap())),
                );
            },
            |_| {},
        )
        .conditions(
            filter.storage.is_some(),
            |q| {
                q.and_where(
                    Expr::col(Storage::StorageId).eq(filter.storage.unwrap()),
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
        .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingId)))
        .expr(Expr::col((Borrowing::Table, Borrowing::BorrowingComment)))
        .order_by(order_by, order)
        .group_by_col((Product::Table, Product::ProductId))
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

    debug!("storages: {:#?}", storages);

    Ok((storages, count))
}
