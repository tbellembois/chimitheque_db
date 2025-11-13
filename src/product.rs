use crate::{
    bookmark::Bookmark,
    borrowing::Borrowing,
    casnumber::CasNumber,
    category::Category,
    cenumber::CeNumber,
    classofcompound::ClassOfCompound,
    empiricalformula::EmpiricalFormula,
    entity::{Entity, EntityWrapper},
    entitypeople::Entitypeople,
    hazardstatement::HazardStatement,
    linearformula::LinearFormula,
    name::Name,
    permission::Permission,
    person::{Person, PersonWrapper},
    physicalstate::PhysicalState,
    precautionarystatement::PrecautionaryStatement,
    producer::Producer,
    producerref::{self, ProducerRef},
    productclassesofcompounds::{Productclassesofcompounds, ProductclassesofcompoundsWrapper},
    producthazardstatements::{Producthazardstatements, ProducthazardstatementsWrapper},
    productprecautionarystatements::{
        Productprecautionarystatements, ProductprecautionarystatementsWrapper,
    },
    productsupplierrefs::{Productsupplierrefs, ProductsupplierrefsWrapper},
    productsymbols::{Productsymbols, ProductsymbolsWrapper},
    productsynonyms::{Productsynonyms, ProductsynonymsWrapper},
    producttags::{Producttags, ProducttagsWrapper},
    searchable,
    signalword::SignalWord,
    storage::Storage,
    storelocation::StoreLocation,
    supplier::Supplier,
    supplierref::{self, SupplierRef},
    symbol::Symbol,
    tag::Tag,
    unit::Unit,
};
use chimitheque_types::{
    casnumber::CasNumber as CasNumberStruct, category::Category as CategoryStruct,
    cenumber::CeNumber as CeNumberStruct,
    classofcompound::ClassOfCompound as ClassOfCompoundStruct,
    empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct, entity::Entity as EntityStruct,
    error::ParseError, linearformula::LinearFormula as LinearFormulaStruct,
    name::Name as NameStruct, person::Person as PersonStruct,
    physicalstate::PhysicalState as PhysicalStateStruct, producer::Producer as ProducerStruct,
    producerref::ProducerRef as ProducerRefStruct, product::Product as ProductStruct,
    producttype::ProductType, requestfilter::RequestFilter,
    signalword::SignalWord as SignalWordStruct, supplierref::SupplierRef as SupplierRefStruct,
    tag::Tag as TagStruct, unit::Unit as UnitStruct, unittype::UnitType,
};
use chimitheque_utils::string::Transform;
use log::debug;
use rusqlite::{Connection, Row, Transaction};
use sea_query::{
    any, Alias, ColumnRef, Cond, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SimpleExpr,
    SqliteQueryBuilder,
};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};
use serde::Serialize;
use std::str::FromStr;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Product {
    Table,
    Person,
    ProductId,
    ProductType,
    ProductInchi,
    ProductInchikey,
    ProductCanonicalSmiles,
    ProductMolecularWeight,
    ProductSpecificity,
    ProductMsds,
    ProductRestricted,
    ProductRadioactive,
    ProductTwodFormula,
    ProductThreedFormula,
    ProductDisposalComment,
    ProductRemark,
    ProductTemperature,
    ProductSheet,
    ProductNumberPerCarton,
    ProductNumberPerBag,
    EmpiricalFormula,
    LinearFormula,
    PhysicalState,
    SignalWord,
    Category,
    Name,
    CasNumber,
    CeNumber,
    UnitMolecularWeight,
    UnitTemperature,
    ProducerRef,
}

#[derive(Debug, Serialize)]
pub struct ProductWrapper(pub ProductStruct);

impl TryFrom<&Row<'_>> for ProductWrapper {
    type Error = ParseError;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let maybe_unit_temperature_type_string: Option<String> =
            row.get_unwrap("unit_temperature_unit_type");
        let maybe_unit_molecular_weight_type_string: Option<String> =
            row.get_unwrap("unit_molecular_weight_unit_type");
        let maybe_cas_number: Option<u64> = row.get_unwrap("cas_number_id");
        let maybe_ce_number: Option<u64> = row.get_unwrap("ce_number_id");
        let maybe_empirical_formula: Option<u64> = row.get_unwrap("empirical_formula_id");
        let maybe_linear_formula: Option<u64> = row.get_unwrap("linear_formula_id");
        let maybe_physical_state: Option<u64> = row.get_unwrap("physical_state_id");
        let maybe_signal_word: Option<u64> = row.get_unwrap("signal_word_id");
        let maybe_category: Option<u64> = row.get_unwrap("category_id");
        let maybe_producer_ref: Option<u64> = row.get_unwrap("producer_ref_id");
        let maybe_unit_temperature: Option<u64> = row.get_unwrap("unit_temperature_unit_id");
        let maybe_unit_molecular_weight: Option<u64> =
            row.get_unwrap("unit_molecular_weight_unit_id");
        let product_type_string: String = row.get_unwrap("product_type");

        // Extract unit temperature type if some.
        let unit_temperature_type: UnitType = if maybe_unit_temperature.is_some() {
            UnitType::from_str(&maybe_unit_temperature_type_string.unwrap())?
        } else {
            Default::default()
        };

        // Extract unit molecular weight type if some.
        let unit_molecular_weight_type: UnitType = if maybe_unit_molecular_weight.is_some() {
            UnitType::from_str(&maybe_unit_molecular_weight_type_string.unwrap())?
        } else {
            Default::default()
        };

        Ok(Self(ProductStruct {
            product_id: row.get_unwrap("product_id"),
            name: NameStruct {
                name_id: row.get_unwrap("name_id"),
                name_label: row.get_unwrap("name_label"),
                ..Default::default()
            },
            person: PersonStruct {
                person_id: row.get_unwrap("person_id"),
                person_email: row.get_unwrap("person_email"),
                ..Default::default()
            },
            cas_number: maybe_cas_number.map(|_| CasNumberStruct {
                cas_number_id: row.get_unwrap("cas_number_id"),
                cas_number_label: row.get_unwrap("cas_number_label"),
                ..Default::default()
            }),
            ce_number: maybe_ce_number.map(|_| CeNumberStruct {
                ce_number_id: row.get_unwrap("ce_number_id"),
                ce_number_label: row.get_unwrap("ce_number_label"),
                ..Default::default()
            }),
            empirical_formula: maybe_empirical_formula.map(|_| EmpiricalFormulaStruct {
                empirical_formula_id: row.get_unwrap("empirical_formula_id"),
                empirical_formula_label: row.get_unwrap("empirical_formula_label"),
                ..Default::default()
            }),
            linear_formula: maybe_linear_formula.map(|_| LinearFormulaStruct {
                linear_formula_id: row.get_unwrap("linear_formula_id"),
                linear_formula_label: row.get_unwrap("linear_formula_label"),
                ..Default::default()
            }),
            physical_state: maybe_physical_state.map(|_| PhysicalStateStruct {
                physical_state_id: row.get_unwrap("physical_state_id"),
                physical_state_label: row.get_unwrap("physical_state_label"),
                ..Default::default()
            }),
            signal_word: maybe_signal_word.map(|_| SignalWordStruct {
                signal_word_id: row.get_unwrap("signal_word_id"),
                signal_word_label: row.get_unwrap("signal_word_label"),
                ..Default::default()
            }),
            category: maybe_category.map(|_| CategoryStruct {
                category_id: row.get_unwrap("category_id"),
                category_label: row.get_unwrap("category_label"),
                ..Default::default()
            }),
            producer_ref: maybe_producer_ref.map(|_| ProducerRefStruct {
                producer_ref_id: row.get_unwrap("producer_ref_id"),
                producer_ref_label: row.get_unwrap("producer_ref_label"),
                producer: ProducerStruct {
                    producer_id: row.get_unwrap("producer_id"),
                    producer_label: row.get_unwrap("producer_label"),
                    ..Default::default()
                },
                ..Default::default()
            }),
            unit_temperature: maybe_unit_temperature.map(|_| UnitStruct {
                unit_id: row.get_unwrap("unit_temperature_unit_id"),
                unit_label: row.get_unwrap("unit_temperature_unit_label"),
                unit_multiplier: row.get_unwrap("unit_temperature_unit_multiplier"),
                unit_type: unit_temperature_type,
                unit: Default::default(),
            }),
            unit_molecular_weight: maybe_unit_molecular_weight.map(|_| UnitStruct {
                unit_id: row.get_unwrap("unit_molecular_weight_unit_id"),
                unit_label: row.get_unwrap("unit_molecular_weight_unit_label"),
                unit_multiplier: row.get_unwrap("unit_molecular_weight_unit_multiplier"),
                unit_type: unit_molecular_weight_type,
                unit: Default::default(),
            }),

            product_type: ProductType::from_str(&product_type_string)?,
            product_inchi: row.get_unwrap("product_inchi"),
            product_inchikey: row.get_unwrap("product_inchikey"),
            product_canonical_smiles: row.get_unwrap("product_canonical_smiles"),
            product_specificity: row.get_unwrap("product_specificity"),
            product_msds: row.get_unwrap("product_msds"),
            product_restricted: row.get_unwrap("product_restricted"),
            product_radioactive: row.get_unwrap("product_radioactive"),
            product_twod_formula: row.get_unwrap("product_twod_formula"),
            product_threed_formula: row.get_unwrap("product_threed_formula"),
            product_disposal_comment: row.get_unwrap("product_disposal_comment"),
            product_remark: row.get_unwrap("product_remark"),
            product_molecular_weight: row.get_unwrap("product_molecular_weight"),
            product_temperature: row.get_unwrap("product_temperature"),
            product_sheet: row.get_unwrap("product_sheet"),
            product_number_per_carton: row.get_unwrap("product_number_per_carton"),
            product_number_per_bag: row.get_unwrap("product_number_per_bag"),
            product_sl: row.get_unwrap("product_sl"),
            product_hs_cmr: row.get_unwrap("product_hs_cmr"),
            product_has_bookmark: row.get_unwrap("product_has_bookmark"),
            ..Default::default()
        }))
    }
}

fn populate_entity_managers(
    db_connection: &Connection,
    entity: &mut EntityStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    let entity_id = entity.entity_id;

    // Create select query.
    let (sql, values) = Query::select()
        .columns([Person::PersonId, Person::PersonEmail])
        .order_by(Person::PersonEmail, Order::Asc)
        .from(Entity::Table)
        //
        // managers
        //
        .join(
            JoinType::InnerJoin,
            Entitypeople::Table,
            Expr::col((Entitypeople::Table, Entitypeople::EntitypeopleEntityId))
                .equals((Entity::Table, Entity::EntityId)),
        )
        .join(
            JoinType::InnerJoin,
            Person::Table,
            Expr::col((Entitypeople::Table, Entitypeople::EntitypeoplePersonId))
                .equals((Person::Table, Person::PersonId)),
        )
        .and_where(Expr::col((Entity::Table, Entity::EntityId)).eq(entity_id))
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql: {}", sql.clone().as_str());
    debug!("values: {:?}", values);

    // Perform select query.
    let mut stmt = db_connection.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |row| Ok(PersonWrapper::from(row)))?;

    // Populate managers.
    let mut people: Vec<chimitheque_types::person::Person> = vec![];
    for row in rows {
        let person_wrapper = row?;
        people.push(chimitheque_types::person::Person {
            person_id: person_wrapper.0.person_id,
            person_email: person_wrapper.0.person_email,
            ..Default::default()
        })
    }

    if !people.is_empty() {
        entity.managers = Some(people);
    } else {
        entity.managers = None;
    }

    Ok(())
}

fn populate_product_availability(
    db_connection: &Connection,
    product: &mut [ProductStruct],
    person_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Entity::EntityId,
                Entity::EntityName,
                Entity::EntityDescription,
            ])
            .order_by(Entity::EntityName, Order::Asc)
            .group_by_col(Entity::EntityId)
            .from(Storage::Table)
            //
            // store location
            //
            .join(
                JoinType::InnerJoin,
                StoreLocation::Table,
                Expr::col((Storage::Table, Storage::StoreLocation))
                    .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
            )
            //
            // entity
            //
            .join(
                JoinType::InnerJoin,
                Entity::Table,
                Expr::col((Entity::Table, Entity::EntityId))
                    .equals((StoreLocation::Table, StoreLocation::Entity)),
            )
            //
            // managers
            //
            .join(
                JoinType::InnerJoin,
                Entitypeople::Table,
                Expr::col((Entitypeople::Table, Entitypeople::EntitypeopleEntityId))
                    .equals((Entity::Table, Entity::EntityId)),
            )
            .join(
                JoinType::InnerJoin,
                Person::Table,
                Expr::col((Entitypeople::Table, Entitypeople::EntitypeoplePersonId))
                    .equals((Person::Table, Person::PersonId)),
            )
            //
            // permissions
            //
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
                    ),
            )
            //
            // product
            //
            .and_where(Expr::col((Storage::Table, Storage::Product)).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| Ok(EntityWrapper::from(row)))?;

        // Populate entities.
        let mut entities: Vec<chimitheque_types::entity::Entity> = vec![];
        for row in rows {
            let entity_wrapper = row?;
            entities.push(chimitheque_types::entity::Entity {
                entity_id: entity_wrapper.0.entity_id,
                entity_name: entity_wrapper.0.entity_name,
                entity_description: entity_wrapper.0.entity_description,
                ..Default::default()
            })
        }

        if !entities.is_empty() {
            for entity in &mut entities {
                populate_entity_managers(db_connection, entity)?;
            }

            product.product_availability = Some(entities);
        } else {
            product.product_availability = None;
        }
    }

    Ok(())
}

fn populate_product_sc(
    db_connection: &Connection,
    product: &mut [ProductStruct],
    person_id: u64,
    total: bool,
    archived: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("person_id:{:?}", person_id);
    debug!("archived:{:?}", archived);

    for product in product.iter_mut() {
        let product_id = product.product_id;

        let (count_sql, count_values) = Query::select()
            .from(Storage::Table)
            .expr(Expr::col((Storage::Table, Storage::StorageId)).count_distinct())
            .join(
                JoinType::Join,
                Product::Table,
                Expr::col((Storage::Table, Storage::Product))
                    .equals((Product::Table, Product::ProductId)),
            )
            //
            // permissions
            //
            .join(
                // storelocation
                JoinType::Join,
                StoreLocation::Table,
                Expr::col((Storage::Table, Storage::StoreLocation))
                    .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
            )
            .join(
                // entity
                JoinType::Join,
                Entity::Table,
                Expr::col((StoreLocation::Table, StoreLocation::Entity))
                    .equals((Entity::Table, Entity::EntityId)),
            )
            .conditions(
                !total,
                |q| {
                    q.join_as(
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
                                    .or(Expr::col((
                                        Alias::new("perm"),
                                        Alias::new("permission_entity"),
                                    ))
                                    .eq(-1)),
                            ),
                    );
                },
                |_| {},
            )
            .conditions(
                archived,
                |q| {
                    q.and_where(Expr::col((Storage::Table, Storage::StorageArchive)).eq(archived));
                },
                |q| {
                    q.and_where(Expr::col((Storage::Table, Storage::StorageArchive)).eq(false));
                },
            )
            .and_where(Expr::col((Storage::Table, Storage::Product)).eq(product_id))
            .and_where(Expr::col((Storage::Table, Storage::Storage)).is_null())
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", count_sql.clone().as_str());
        debug!("values: {:?}", count_values);

        // Perform count query.
        let mut stmt = db_connection.prepare(count_sql.as_str())?;
        let mut rows = stmt.query(&*count_values.as_params())?;
        let count: u64 = if let Some(row) = rows.next()? {
            row.get_unwrap(0)
        } else {
            0
        };

        // Populate count.
        if total {
            product.product_tsc = Some(count);
        } else if archived {
            product.product_asc = Some(count);
        } else {
            product.product_sc = Some(count);
        }
    }

    Ok(())
}

fn populate_synonyms(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Productsynonyms::ProductsynonymsProductId,
                Productsynonyms::ProductsynonymsNameId,
            ])
            .column(Name::NameLabel)
            .from(Productsynonyms::Table)
            //
            // name
            //
            .join(
                JoinType::LeftJoin,
                Name::Table,
                Expr::col((
                    Productsynonyms::Table,
                    Productsynonyms::ProductsynonymsNameId,
                ))
                .equals((Name::Table, Name::NameId)),
            )
            .and_where(Expr::col(Productsynonyms::ProductsynonymsProductId).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductsynonymsWrapper::from(row))
        })?;

        // Populate product synonyms.
        let mut synonyms: Vec<chimitheque_types::name::Name> = vec![];
        for row in rows {
            let product_synonym_wrapper = row?;
            synonyms.push(chimitheque_types::name::Name {
                match_exact_search: false,
                name_id: Some(product_synonym_wrapper.0.productsynonyms_name_id),
                name_label: product_synonym_wrapper.0.productsynonyms_name_label,
            });
        }

        if !synonyms.is_empty() {
            product.synonyms = Some(synonyms);
        } else {
            product.synonyms = None;
        }
    }

    Ok(())
}

fn populate_classes_of_compound(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Productclassesofcompounds::ProductclassesofcompoundsProductId,
                Productclassesofcompounds::ProductclassesofcompoundsClassOfCompoundId,
            ])
            .column(ClassOfCompound::ClassOfCompoundLabel)
            .from(Productclassesofcompounds::Table)
            //
            // classofcompounds
            //
            .join(
                JoinType::LeftJoin,
                ClassOfCompound::Table,
                Expr::col((
                    Productclassesofcompounds::Table,
                    Productclassesofcompounds::ProductclassesofcompoundsClassOfCompoundId,
                ))
                .equals((ClassOfCompound::Table, ClassOfCompound::ClassOfCompoundId)),
            )
            .and_where(
                Expr::col(Productclassesofcompounds::ProductclassesofcompoundsProductId)
                    .eq(product_id),
            )
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductclassesofcompoundsWrapper::from(row))
        })?;

        // Populate product classes of compound.
        let mut classes_of_compound: Vec<chimitheque_types::classofcompound::ClassOfCompound> =
            vec![];
        for row in rows {
            let product_class_of_compound_wrapper = row?;
            classes_of_compound.push(chimitheque_types::classofcompound::ClassOfCompound {
                match_exact_search: false,
                class_of_compound_id: Some(
                    product_class_of_compound_wrapper
                        .0
                        .productclassesofcompounds_class_of_compound_id,
                ),
                class_of_compound_label: product_class_of_compound_wrapper
                    .0
                    .productclassesofcompounds_class_of_compound_label,
            });
        }

        if !classes_of_compound.is_empty() {
            product.classes_of_compound = Some(classes_of_compound);
        } else {
            product.classes_of_compound = None;
        }
    }

    Ok(())
}

fn populate_symbols(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Productsymbols::ProductsymbolsProductId,
                Productsymbols::ProductsymbolsSymbolId,
            ])
            .column(Symbol::SymbolLabel)
            .from(Productsymbols::Table)
            //
            // symbols
            //
            .join(
                JoinType::LeftJoin,
                Symbol::Table,
                Expr::col((
                    Productsymbols::Table,
                    Productsymbols::ProductsymbolsSymbolId,
                ))
                .equals((Symbol::Table, Symbol::SymbolId)),
            )
            .and_where(Expr::col(Productsymbols::ProductsymbolsProductId).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductsymbolsWrapper::from(row))
        })?;

        // Populate product symbols.
        let mut symbols: Vec<chimitheque_types::symbol::Symbol> = vec![];
        for row in rows {
            let product_symbols_wrapper = row?;
            symbols.push(chimitheque_types::symbol::Symbol {
                match_exact_search: false,
                symbol_id: Some(product_symbols_wrapper.0.productsymbols_symbol_id),
                symbol_label: product_symbols_wrapper.0.productsymbols_symbol_label,
            });
        }

        if !symbols.is_empty() {
            product.symbols = Some(symbols);
        } else {
            product.symbols = None;
        }
    }

    Ok(())
}

fn populate_hazard_statements(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Producthazardstatements::ProducthazardstatementsProductId,
                Producthazardstatements::ProducthazardstatementsHazardStatementId,
            ])
            .column(HazardStatement::HazardStatementLabel)
            .column(HazardStatement::HazardStatementReference)
            .column(HazardStatement::HazardStatementCmr)
            .from(Producthazardstatements::Table)
            //
            // hazardstatement
            //
            .join(
                JoinType::LeftJoin,
                HazardStatement::Table,
                Expr::col((
                    Producthazardstatements::Table,
                    Producthazardstatements::ProducthazardstatementsHazardStatementId,
                ))
                .equals((HazardStatement::Table, HazardStatement::HazardStatementId)),
            )
            .and_where(
                Expr::col(Producthazardstatements::ProducthazardstatementsProductId).eq(product_id),
            )
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProducthazardstatementsWrapper::from(row))
        })?;

        // Populate product hazard statements.
        let mut hazard_statements: Vec<chimitheque_types::hazardstatement::HazardStatement> =
            vec![];
        for row in rows {
            let product_hazardstatements_wrapper = row?;
            hazard_statements.push(chimitheque_types::hazardstatement::HazardStatement {
                match_exact_search: false,
                hazard_statement_id: Some(
                    product_hazardstatements_wrapper
                        .0
                        .producthazardstatements_hazard_statement_id,
                ),
                hazard_statement_label: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazard_statement_label,
                hazard_statement_reference: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazard_statement_reference,
                hazard_statement_cmr: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazard_statement_cmr,
            });
        }

        if !hazard_statements.is_empty() {
            product.hazard_statements = Some(hazard_statements);
        } else {
            product.hazard_statements = None;
        }
    }

    Ok(())
}

fn populate_precautionary_statements(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Productprecautionarystatements::ProductprecautionarystatementsProductId,
                Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId,
            ])
            .column(PrecautionaryStatement::PrecautionaryStatementLabel)
            .column(PrecautionaryStatement::PrecautionaryStatementReference)
            .from(Productprecautionarystatements::Table)
            //
            // precautionarystatement
            //
            .join(
                JoinType::LeftJoin,
                PrecautionaryStatement::Table,
                Expr::col((
                    Productprecautionarystatements::Table,
                    Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId,
                ))
                .equals((PrecautionaryStatement::Table, PrecautionaryStatement::PrecautionaryStatementId)),
            )
            .and_where(
                Expr::col(Productprecautionarystatements::ProductprecautionarystatementsProductId).eq(product_id),
            )
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductprecautionarystatementsWrapper::from(row))
        })?;

        // Populate product precautionary statements.
        let mut precautionary_statements: Vec<
            chimitheque_types::precautionarystatement::PrecautionaryStatement,
        > = vec![];
        for row in rows {
            let product_precautionarystatements_wrapper = row?;
            precautionary_statements.push(
                chimitheque_types::precautionarystatement::PrecautionaryStatement {
                    match_exact_search: false,
                    precautionary_statement_id: Some(
                        product_precautionarystatements_wrapper
                            .0
                            .productprecautionarystatements_precautionary_statement_id,
                    ),
                    precautionary_statement_label: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionary_statement_label,
                    precautionary_statement_reference: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionary_statement_reference,
                },
            );
        }

        if !precautionary_statements.is_empty() {
            product.precautionary_statements = Some(precautionary_statements);
        } else {
            product.precautionary_statements = None;
        }
    }

    Ok(())
}

fn populate_supplier_refs(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Productsupplierrefs::ProductsupplierrefsProductId,
                Productsupplierrefs::ProductsupplierrefsSupplierRefId,
            ])
            .columns([SupplierRef::SupplierRefId, SupplierRef::SupplierRefLabel])
            .columns([Supplier::SupplierId, Supplier::SupplierLabel])
            .from(Productsupplierrefs::Table)
            //
            // supplierref
            //
            .join(
                JoinType::LeftJoin,
                SupplierRef::Table,
                Expr::col((
                    Productsupplierrefs::Table,
                    Productsupplierrefs::ProductsupplierrefsSupplierRefId,
                ))
                .equals((SupplierRef::Table, SupplierRef::SupplierRefId)),
            )
            //
            // supplier
            //
            .join(
                JoinType::LeftJoin,
                Supplier::Table,
                Expr::col((SupplierRef::Table, SupplierRef::Supplier))
                    .equals((Supplier::Table, Supplier::SupplierId)),
            )
            .and_where(Expr::col(Productsupplierrefs::ProductsupplierrefsProductId).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductsupplierrefsWrapper::from(row))
        })?;

        // Populate product supplier refs.
        let mut supplier_refs: Vec<chimitheque_types::supplierref::SupplierRef> = vec![];
        for row in rows {
            let product_supplierrefs_wrapper = row?;
            supplier_refs.push(chimitheque_types::supplierref::SupplierRef {
                match_exact_search: false,
                supplier_ref_id: Some(
                    product_supplierrefs_wrapper
                        .0
                        .productsupplierrefs_supplier_ref_id,
                ),
                supplier_ref_label: product_supplierrefs_wrapper
                    .0
                    .productsupplierrefs_supplier_ref_label,
                supplier: chimitheque_types::supplier::Supplier {
                    match_exact_search: false,
                    supplier_id: Some(
                        product_supplierrefs_wrapper
                            .0
                            .productsupplierrefs_supplier_id,
                    ),
                    supplier_label: product_supplierrefs_wrapper
                        .0
                        .productsupplierrefs_supplier_label,
                },
            });
        }

        if !supplier_refs.is_empty() {
            product.supplier_refs = Some(supplier_refs);
        } else {
            product.supplier_refs = None;
        }
    }

    Ok(())
}

fn populate_tags(
    db_connection: &Connection,
    product: &mut [ProductStruct],
) -> Result<(), Box<dyn std::error::Error>> {
    for product in product.iter_mut() {
        let product_id = product.product_id;

        // Create select query.
        let (sql, values) = Query::select()
            .columns([
                Producttags::ProducttagsProductId,
                Producttags::ProducttagsTagId,
            ])
            .column(Tag::TagLabel)
            .from(Producttags::Table)
            //
            // tags
            //
            .join(
                JoinType::LeftJoin,
                Tag::Table,
                Expr::col((Producttags::Table, Producttags::ProducttagsTagId))
                    .equals((Tag::Table, Tag::TagId)),
            )
            .and_where(Expr::col(Producttags::ProducttagsProductId).eq(product_id))
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProducttagsWrapper::from(row))
        })?;

        // Populate product tags.
        let mut tags: Vec<chimitheque_types::tag::Tag> = vec![];
        for row in rows {
            let product_tags_wrapper = row?;
            tags.push(chimitheque_types::tag::Tag {
                match_exact_search: false,
                tag_id: Some(product_tags_wrapper.0.producttags_tag_id),
                tag_label: product_tags_wrapper.0.producttags_tag_label,
            });
        }

        if !tags.is_empty() {
            product.tags = Some(tags);
        } else {
            product.tags = None;
        }
    }

    Ok(())
}

pub fn get_products(
    db_connection: &Connection,
    filter: RequestFilter,
    person_id: u64,
) -> Result<(Vec<ProductStruct>, usize), Box<dyn std::error::Error>> {
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

    // Order_by and Order.
    let order_by: ColumnRef = if let Some(order_by_string) = filter.order_by {
        match order_by_string.as_str() {
            "name" => (Name::Table, Name::NameLabel).into_column_ref(),
            "empirical_formula" => Product::EmpiricalFormula.into_column_ref(),
            "cas_number" => (CasNumber::Table, CasNumber::CasNumberLabel).into_column_ref(),
            _ => (Product::Table, Product::ProductId).into_column_ref(),
        }
    } else {
        (Product::Table, Product::ProductId).into_column_ref()
    };

    let order = if filter.order.eq_ignore_ascii_case("desc") {
        Order::Desc
    } else {
        Order::Asc
    };

    // Create common query statement.
    let mut expression = Query::select();
    expression
        .from(Product::Table)
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
            Expr::col((Product::Table, Product::Person)).equals((Person::Table, Person::PersonId)),
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
        // linear formula
        //
        .join(
            JoinType::LeftJoin,
            LinearFormula::Table,
            Expr::col((Product::Table, Product::LinearFormula))
                .equals((LinearFormula::Table, LinearFormula::LinearFormulaId)),
        )
        //
        // physical state
        //
        .join(
            JoinType::LeftJoin,
            PhysicalState::Table,
            Expr::col((Product::Table, Product::PhysicalState))
                .equals((PhysicalState::Table, PhysicalState::PhysicalStateId)),
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
        // unit temperature
        //
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("unit_temperature"),
            Expr::col((Product::Table, Product::UnitTemperature))
                .equals((Alias::new("unit_temperature"), Unit::UnitId)),
        )
        //
        // unit molecular weight
        //
        .join_as(
            JoinType::LeftJoin,
            Unit::Table,
            Alias::new("unit_molecular_weight"),
            Expr::col((Product::Table, Product::UnitMolecularWeight))
                .equals((Alias::new("unit_molecular_weight"), Unit::UnitId)),
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
        // bookmarks
        //
        // .join(
        //     JoinType::LeftJoin,
        //     Bookmark::Table,
        //     Expr::col((Bookmark::Table, Bookmark::Product)).equals((
        //         Product::Table,
        //         Product::ProductId,
        //     )).and(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id)
        // ))
        //
        // storage -> permissions
        //
        .join(
            JoinType::LeftJoin,
            Storage::Table,
            Expr::col((Storage::Table, Storage::Product))
                .equals((Product::Table, Product::ProductId)),
        )
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
                        .is_in(["all", "products"]),
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
                            Expr::col((Product::Table, Product::ProductId))
                            .eq(filter.id.unwrap()),
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
            |q| {
                q.join(
                    // bookmark
                    JoinType::LeftJoin,
                    Bookmark::Table,
                    Expr::col((Bookmark::Table, Bookmark::Product))
                    .equals((Product::Table, Product::ProductId)),
                );
                // q.and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id));
            },)
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
        .expr(Expr::col((Product::Table, Product::ProductId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
            Product::ProductType,
            Product::ProductId,
            Product::ProductInchi,
            Product::ProductInchikey,
            Product::ProductCanonicalSmiles,
            Product::ProductMolecularWeight,
            Product::ProductSpecificity,
            Product::ProductMsds,
            Product::ProductRestricted,
            Product::ProductRadioactive,
            Product::ProductTwodFormula,
            Product::ProductThreedFormula,
            Product::ProductDisposalComment,
            Product::ProductRemark,
            Product::ProductMolecularWeight,
            Product::ProductTemperature,
            Product::ProductSheet,
            Product::ProductNumberPerBag,
            Product::ProductNumberPerCarton,
        ])
        .expr(Expr::col((Name::Table, Name::NameId)))
        .expr(Expr::col((Name::Table, Name::NameLabel)))
        .expr(Expr::col((Person::Table, Person::PersonId)))
        .expr(Expr::col((Person::Table, Person::PersonEmail)))
        .expr(Expr::col((CasNumber::Table, CasNumber::CasNumberId)))
        .expr(Expr::col((CasNumber::Table, CasNumber::CasNumberLabel)))
        .expr(Expr::col((CeNumber::Table, CeNumber::CeNumberId)))
        .expr(Expr::col((CeNumber::Table, CeNumber::CeNumberLabel)))
        .expr(Expr::col((
            EmpiricalFormula::Table,
            EmpiricalFormula::EmpiricalFormulaId,
        )))
        .expr(Expr::col((
            EmpiricalFormula::Table,
            EmpiricalFormula::EmpiricalFormulaLabel,
        )))
        .expr(Expr::col((
            LinearFormula::Table,
            LinearFormula::LinearFormulaId,
        )))
        .expr(Expr::col((
            LinearFormula::Table,
            LinearFormula::LinearFormulaLabel,
        )))
        .expr(Expr::col((
            PhysicalState::Table,
            PhysicalState::PhysicalStateId,
        )))
        .expr(Expr::col((
            PhysicalState::Table,
            PhysicalState::PhysicalStateLabel,
        )))
        .expr(Expr::col((Category::Table, Category::CategoryId)))
        .expr(Expr::col((Category::Table, Category::CategoryLabel)))
        .expr(Expr::col((SignalWord::Table, SignalWord::SignalWordId)))
        .expr(Expr::col((SignalWord::Table, SignalWord::SignalWordLabel)))
        .expr(Expr::col((ProducerRef::Table, ProducerRef::ProducerRefId)))
        .expr(Expr::col((
            ProducerRef::Table,
            ProducerRef::ProducerRefLabel,
        )))
        .expr(Expr::col((Producer::Table, Producer::ProducerId)))
        .expr(Expr::col((Producer::Table, Producer::ProducerLabel)))
        .expr_as(
            Expr::case(
                           Expr::exists(
                               Query::select()
                                   .expr(Expr::col((Bookmark::Table, Bookmark::BookmarkId)))
                                   .from(Bookmark::Table)
                                   .and_where(
                                       Expr::col((Bookmark::Table, Bookmark::Product))
                                           .equals((Product::Table, Product::ProductId)),
                                   )
                                   .and_where(Expr::col((Bookmark::Table, Bookmark::Person)).eq(person_id))
                                   .take(),
                           ),
                           Expr::val(true),
                       )
                       .finally(Expr::val(false))
            , Alias::new("product_has_bookmark"))
        .expr_as(
            Expr::col((Alias::new("unit_temperature"), Alias::new("unit_id"))),
            Alias::new("unit_temperature_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_temperature"), Alias::new("unit_label"))),
            Alias::new("unit_temperature_unit_label"),
        )
        .expr_as(
            Expr::col((
                Alias::new("unit_temperature"),
                Alias::new("unit_multiplier"),
            )),
            Alias::new("unit_temperature_unit_multiplier"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_temperature"), Alias::new("unit_type"))),
            Alias::new("unit_temperature_unit_type"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_molecular_weight"), Alias::new("unit_id"))),
            Alias::new("unit_molecular_weight_unit_id"),
        )
        .expr_as(
            Expr::col((
                Alias::new("unit_molecular_weight"),
                Alias::new("unit_label"),
            )),
            Alias::new("unit_molecular_weight_unit_label"),
        )
        .expr_as(
            Expr::col((
                Alias::new("unit_molecular_weight"),
                Alias::new("unit_multiplier"),
            )),
            Alias::new("unit_molecular_weight_unit_multiplier"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_molecular_weight"), Alias::new("unit_type"))),
            Alias::new("unit_molecular_weight_unit_type"),
        )
        .expr_as(
            Expr::cust("GROUP_CONCAT(DISTINCT REGEXP_SUBSTR(storage.storage_barecode, '[a-zA-Z]{1}[0-9]+.'))"),
            Alias::new("product_sl"),
        )
        .expr_as(
            Expr::cust("GROUP_CONCAT(DISTINCT hazard_statement.hazard_statement_cmr)"),
                 Alias::new("product_hs_cmr"),
        )
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
    let mut products = Vec::new();
    let mut rows = stmt.query(&*select_values.as_params())?;
    while let Some(row) = rows.next()? {
        let product = ProductWrapper::try_from(row)?;
        products.push(product.0);
    }

    populate_synonyms(db_connection, &mut products)?;
    populate_classes_of_compound(db_connection, &mut products)?;
    populate_symbols(db_connection, &mut products)?;
    populate_hazard_statements(db_connection, &mut products)?;
    populate_precautionary_statements(db_connection, &mut products)?;
    populate_supplier_refs(db_connection, &mut products)?;
    populate_tags(db_connection, &mut products)?;
    populate_product_availability(db_connection, &mut products, person_id)?;

    populate_product_sc(db_connection, &mut products, person_id, false, false)?;
    populate_product_sc(db_connection, &mut products, person_id, true, false)?;
    populate_product_sc(db_connection, &mut products, person_id, false, true)?;

    debug!("products: {:#?}", products);

    Ok((products, count))
}

pub fn create_update_product(
    db_connection: &mut Connection,
    mut product: ProductStruct,
) -> Result<u64, Box<dyn std::error::Error>> {
    debug!("create_update_product: {:#?}", product);

    let db_transaction = db_connection.transaction()?;

    //
    // name
    //
    let name = product.name.clone();
    if name.name_id.is_none() {
        let name_id = searchable::create_update(
            &NameStruct {
                ..Default::default()
            },
            None,
            &db_transaction,
            name.name_label.as_str(),
            Transform::None,
        )?;
        product.name = NameStruct {
            name_id: Some(name_id),
            name_label: name.name_label,
            ..Default::default()
        };
    }

    //
    // cas number
    //
    if let Some(cas_number) = product.cas_number.clone() {
        if cas_number.cas_number_id.is_none() {
            let cas_number_id = searchable::create_update(
                &CasNumberStruct {
                    ..Default::default()
                },
                None,
                &db_transaction,
                cas_number.cas_number_label.as_str(),
                Transform::None,
            )?;
            product.cas_number = Some(CasNumberStruct {
                cas_number_id: Some(cas_number_id),
                cas_number_label: cas_number.cas_number_label,
                ..Default::default()
            });
        }
    }

    //
    // ce number
    //
    if let Some(ce_number) = product.ce_number.clone() {
        if ce_number.ce_number_id.is_none() {
            let ce_number_id = searchable::create_update(
                &CeNumberStruct {
                    ..Default::default()
                },
                None,
                &db_transaction,
                ce_number.ce_number_label.as_str(),
                Transform::None,
            )?;
            product.ce_number = Some(CeNumberStruct {
                ce_number_id: Some(ce_number_id),
                ce_number_label: ce_number.ce_number_label,
                ..Default::default()
            });
        }
    }

    //
    // empirical formula
    //
    if let Some(empirical_formula) = product.empirical_formula.clone() {
        if empirical_formula.empirical_formula_id.is_none() {
            let empirical_formula_id = searchable::create_update(
                &EmpiricalFormulaStruct {
                    ..Default::default()
                },
                None,
                &db_transaction,
                empirical_formula.empirical_formula_label.as_str(),
                Transform::None,
            )?;
            product.empirical_formula = Some(EmpiricalFormulaStruct {
                empirical_formula_id: Some(empirical_formula_id),
                empirical_formula_label: empirical_formula.empirical_formula_label,
                ..Default::default()
            });
        }
    }

    //
    // linear formula
    //
    if let Some(linear_formula) = product.linear_formula.clone() {
        if linear_formula.linear_formula_id.is_none() {
            let linear_formula_id = searchable::create_update(
                &LinearFormulaStruct {
                    ..Default::default()
                },
                None,
                &db_transaction,
                linear_formula.linear_formula_label.as_str(),
                Transform::None,
            )?;
            product.linear_formula = Some(LinearFormulaStruct {
                linear_formula_id: Some(linear_formula_id),
                linear_formula_label: linear_formula.linear_formula_label,
                ..Default::default()
            });
        }
    }

    //
    // category
    //
    if let Some(category) = product.category.clone() {
        if category.category_id.is_none() {
            let category_id = searchable::create_update(
                &CategoryStruct {
                    ..Default::default()
                },
                None,
                &db_transaction,
                category.category_label.as_str(),
                Transform::None,
            )?;
            product.category = Some(CategoryStruct {
                category_id: Some(category_id),
                category_label: category.category_label,
                ..Default::default()
            });
        }
    }

    //
    // producer reference
    //
    if let Some(producer_ref) = product.producer_ref.clone() {
        if producer_ref.producer_ref_id.is_none() {
            let producer_ref_id = Some(producerref::create_update_producer_ref(
                &db_transaction,
                &producer_ref,
            )?);
            product.producer_ref = Some(ProducerRefStruct {
                producer_ref_id,
                producer_ref_label: producer_ref.producer_ref_label,
                ..Default::default()
            });
        }
    }

    //
    // synonyms
    //
    if let Some(synonyms) = product.synonyms {
        let mut product_synonyms: Vec<NameStruct> = Vec::new();

        for name in synonyms {
            let mut name_id = name.name_id;
            if name_id.is_none() {
                name_id = Some(searchable::create_update(
                    &NameStruct {
                        ..Default::default()
                    },
                    None,
                    &db_transaction,
                    name.name_label.as_str(),
                    Transform::None,
                )?);
            }
            product_synonyms.push(NameStruct {
                name_id,
                name_label: name.name_label,
                ..Default::default()
            });
        }

        product.synonyms = Some(product_synonyms);
    }

    //
    // classes of compound
    //
    if let Some(classes_of_compound) = product.classes_of_compound {
        let mut product_classes_of_compound: Vec<ClassOfCompoundStruct> = Vec::new();

        for class_of_compound in classes_of_compound {
            let mut class_of_compound_id = class_of_compound.class_of_compound_id;
            if class_of_compound_id.is_none() {
                class_of_compound_id = Some(searchable::create_update(
                    &ClassOfCompoundStruct {
                        ..Default::default()
                    },
                    None,
                    &db_transaction,
                    class_of_compound.class_of_compound_label.as_str(),
                    Transform::None,
                )?);
            }
            product_classes_of_compound.push(ClassOfCompoundStruct {
                class_of_compound_id,
                class_of_compound_label: class_of_compound.class_of_compound_label,
                ..Default::default()
            });
        }

        product.classes_of_compound = Some(product_classes_of_compound);
    }

    //
    // supppliers references
    //
    if let Some(supplier_refs) = product.supplier_refs {
        let mut product_supplier_refs: Vec<SupplierRefStruct> = Vec::new();

        for supplier_ref in supplier_refs {
            let mut supplier_ref_id = supplier_ref.supplier_ref_id;
            if supplier_ref_id.is_none() {
                supplier_ref_id = Some(supplierref::create_update_supplier_ref(
                    &db_transaction,
                    &supplier_ref,
                )?);
            }
            product_supplier_refs.push(SupplierRefStruct {
                supplier_ref_id,
                supplier_ref_label: supplier_ref.supplier_ref_label,
                ..Default::default()
            })
        }

        product.supplier_refs = Some(product_supplier_refs);
    }

    //
    // tags
    //
    if let Some(tags) = product.tags {
        let mut product_tags: Vec<TagStruct> = Vec::new();

        for tag in tags {
            let mut tag_id = tag.tag_id;
            if tag_id.is_none() {
                tag_id = Some(searchable::create_update(
                    &TagStruct {
                        ..Default::default()
                    },
                    None,
                    &db_transaction,
                    tag.tag_label.as_str(),
                    Transform::None,
                )?);
            }
            product_tags.push(TagStruct {
                tag_id,
                tag_label: tag.tag_label,
                ..Default::default()
            });
        }

        product.tags = Some(product_tags);
    }

    // Update request: list of (columns, values) pairs to insert.
    // let mut columns_values = vec![
    //     (
    //         Product::ProductType,
    //         product.product_type.to_string().into(),
    //     ),
    //     (
    //         Product::ProductRestricted,
    //         product.product_restricted.into(),
    //     ),
    //     (
    //         Product::ProductRadioactive,
    //         product.product_radioactive.into(),
    //     ),
    //     (Product::Name, product.name.name_id.into()),
    //     (Product::Person, product.person.person_id.into()),
    // ];

    // Create request: list of columns and values to insert.
    let mut columns = vec![
        Product::ProductType,
        Product::ProductRestricted,
        Product::ProductRadioactive,
        Product::Name,
        Product::Person,
    ];
    let mut values = vec![
        SimpleExpr::Value(product.product_type.to_string().into()),
        SimpleExpr::Value(product.product_restricted.into()),
        SimpleExpr::Value(product.product_radioactive.into()),
        SimpleExpr::Value(product.name.name_id.into()),
        SimpleExpr::Value(product.person.person_id.into()),
    ];

    if let Some(inchi) = &product.product_inchi {
        // columns_values.push((Product::ProductInchi, inchi.clone().into()));

        columns.push(Product::ProductInchi);
        values.push(SimpleExpr::Value(inchi.into()));
    }

    if let Some(inchikey) = &product.product_inchikey {
        // columns_values.push((Product::ProductInchikey, inchikey.clone().into()));

        columns.push(Product::ProductInchikey);
        values.push(SimpleExpr::Value(inchikey.into()));
    }

    if let Some(canonical_smiles) = &product.product_canonical_smiles {
        // columns_values.push((
        //     Product::ProductCanonicalSmiles,
        //     canonical_smiles.clone().into(),
        // ));

        columns.push(Product::ProductCanonicalSmiles);
        values.push(SimpleExpr::Value(canonical_smiles.into()));
    }

    if let Some(specificity) = &product.product_specificity {
        // columns_values.push((Product::ProductSpecificity, specificity.clone().into()));

        columns.push(Product::ProductSpecificity);
        values.push(SimpleExpr::Value(specificity.into()));
    }

    if let Some(msds) = &product.product_msds {
        // columns_values.push((Product::ProductMsds, msds.clone().into()));

        columns.push(Product::ProductMsds);
        values.push(SimpleExpr::Value(msds.into()));
    }

    if let Some(twod_formula) = &product.product_twod_formula {
        // columns_values.push((Product::ProductTwodFormula, twod_formula.clone().into()));

        columns.push(Product::ProductTwodFormula);
        values.push(SimpleExpr::Value(twod_formula.into()));
    }

    if let Some(threed_formula) = &product.product_threed_formula {
        // columns_values.push((Product::ProductThreedFormula, threed_formula.clone().into()));

        columns.push(Product::ProductThreedFormula);
        values.push(SimpleExpr::Value(threed_formula.into()));
    }

    if let Some(disposal_comment) = &product.product_disposal_comment {
        // columns_values.push((
        //     Product::ProductDisposalComment,
        //     disposal_comment.clone().into(),
        // ));

        columns.push(Product::ProductDisposalComment);
        values.push(SimpleExpr::Value(disposal_comment.into()));
    }

    if let Some(remark) = &product.product_remark {
        // columns_values.push((Product::ProductRemark, remark.clone().into()));

        columns.push(Product::ProductRemark);
        values.push(SimpleExpr::Value(remark.into()));
    }

    if let Some(molecular_weight) = &product.product_molecular_weight {
        // columns_values.push((
        //     Product::ProductMolecularWeight,
        //     molecular_weight.to_owned().into(),
        // ));

        columns.push(Product::ProductMolecularWeight);
        values.push(SimpleExpr::Value(molecular_weight.to_owned().into()));
    }

    if let Some(temperature) = &product.product_temperature {
        // columns_values.push((Product::ProductTemperature, temperature.to_owned().into()));

        columns.push(Product::ProductTemperature);
        values.push(SimpleExpr::Value(temperature.to_owned().into()));
    }

    if let Some(sheet) = &product.product_sheet {
        // columns_values.push((Product::ProductSheet, sheet.clone().into()));

        columns.push(Product::ProductSheet);
        values.push(SimpleExpr::Value(sheet.into()));
    }

    if let Some(number_per_carton) = &product.product_number_per_carton {
        // columns_values.push((
        //     Product::ProductNumberPerCarton,
        //     number_per_carton.to_owned().into(),
        // ));

        columns.push(Product::ProductNumberPerCarton);
        values.push(SimpleExpr::Value(number_per_carton.to_owned().into()));
    }

    if let Some(number_per_bag) = &product.product_number_per_bag {
        // columns_values.push((
        //     Product::ProductNumberPerBag,
        //     number_per_bag.to_owned().into(),
        // ));

        columns.push(Product::ProductNumberPerBag);
        values.push(SimpleExpr::Value(number_per_bag.to_owned().into()));
    }

    // --

    if let Some(empirical_formula) = &product.empirical_formula {
        // columns_values.push((
        //     Product::EmpiricalFormula,
        //     empirical_formula.empirical_formula_id.into(),
        // ));

        columns.push(Product::EmpiricalFormula);
        values.push(SimpleExpr::Value(
            empirical_formula.empirical_formula_id.into(),
        ));
    }

    if let Some(linear_formula) = &product.linear_formula {
        // columns_values.push((
        //     Product::LinearFormula,
        //     linear_formula.linear_formula_id.into(),
        // ));

        columns.push(Product::LinearFormula);
        values.push(SimpleExpr::Value(linear_formula.linear_formula_id.into()));
    }

    if let Some(physical_state) = &product.physical_state {
        // columns_values.push((
        //     Product::PhysicalState,
        //     physical_state.physical_state_id.into(),
        // ));

        columns.push(Product::PhysicalState);
        values.push(SimpleExpr::Value(physical_state.physical_state_id.into()));
    }

    if let Some(cas_number) = &product.cas_number {
        // columns_values.push((Product::CasNumber, cas_number.cas_number_id.into()));

        columns.push(Product::CasNumber);
        values.push(SimpleExpr::Value(cas_number.cas_number_id.into()));
    }

    if let Some(ce_number) = &product.ce_number {
        // columns_values.push((Product::CeNumber, ce_number.ce_number_id.into()));

        columns.push(Product::CeNumber);
        values.push(SimpleExpr::Value(ce_number.ce_number_id.into()));
    }

    if let Some(producer_ref) = &product.producer_ref {
        // columns_values.push((Product::ProducerRef, producer_ref.producer_ref_id.into()));

        columns.push(Product::ProducerRef);
        values.push(SimpleExpr::Value(producer_ref.producer_ref_id.into()));
    }

    if let Some(category) = &product.category {
        // columns_values.push((Product::Category, category.category_id.into()));

        columns.push(Product::Category);
        values.push(SimpleExpr::Value(category.category_id.into()));
    }

    if let Some(signal_word) = &product.signal_word {
        // columns_values.push((Product::SignalWord, signal_word.signal_word_id.into()));

        columns.push(Product::SignalWord);
        values.push(SimpleExpr::Value(signal_word.signal_word_id.into()));
    }

    if let Some(unit_temperature) = &product.unit_temperature {
        // columns_values.push((Product::UnitTemperature, unit_temperature.unit_id.into()));

        columns.push(Product::UnitTemperature);
        values.push(SimpleExpr::Value(unit_temperature.unit_id.into()));
    }

    if let Some(unit_molecular_weight) = &product.unit_molecular_weight {
        // columns_values.push((
        //     Product::UnitMolecularWeight,
        //     unit_molecular_weight.unit_id.into(),
        // ));

        columns.push(Product::UnitMolecularWeight);
        values.push(SimpleExpr::Value(unit_molecular_weight.unit_id.into()));
    }

    // --

    let sql_query: String;
    let sql_values: RusqliteValues = RusqliteValues(vec![]);

    if let Some(product_id) = product.product_id {
        // Update query.
        // (sql_query, sql_values) = Query::update()
        //     .table(Product::Table)
        //     .values(columns_values)
        //     .and_where(Expr::col(Product::ProductId).eq(product_id))
        //     .build_rusqlite(SqliteQueryBuilder);
        columns.push(Product::ProductId);
        values.push(SimpleExpr::Value(product_id.into()));

        sql_query = Query::insert()
            .replace()
            .into_table(Product::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    } else {
        // Insert query.
        sql_query = Query::insert()
            .into_table(Product::Table)
            .columns(columns)
            .values(values)?
            .to_string(SqliteQueryBuilder);
    }

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;

    let last_insert_update_id: u64;

    if let Some(product_id) = product.product_id {
        last_insert_update_id = product_id;
    } else {
        last_insert_update_id = db_transaction.last_insert_rowid().try_into()?;
        product.product_id = Some(last_insert_update_id)
    }

    debug!("last_insert_update_id: {}", last_insert_update_id);

    // --

    create_update_product_classes_of_compound(&db_transaction, &product)?;
    create_update_product_synonyms(&db_transaction, &product)?;
    create_update_product_symbols(&db_transaction, &product)?;
    create_update_product_precautionary_statements(&db_transaction, &product)?;
    create_update_product_hazard_statements(&db_transaction, &product)?;
    create_update_product_supplier_refs(&db_transaction, &product)?;
    create_update_product_tags(&db_transaction, &product)?;

    db_transaction.commit()?;

    Ok(last_insert_update_id)
}

fn create_update_product_symbols(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_symbols: {:#?}", product);

    let mut product_symbols_ids: Vec<u64> = Vec::new();

    if let Some(symbols_ids) = &product.symbols {
        product_symbols_ids = symbols_ids
            .iter()
            .map(|s| s.symbol_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Productsymbols::Table)
        .cond_where(
            Cond::all()
                .add(Expr::col(Productsymbols::ProductsymbolsProductId).eq(product.product_id))
                .add(
                    Expr::col(Productsymbols::ProductsymbolsSymbolId)
                        .is_not_in(product_symbols_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for name_id in product_symbols_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Productsymbols::Table)
            .columns([
                Productsymbols::ProductsymbolsProductId,
                Productsymbols::ProductsymbolsSymbolId,
            ])
            .values([product.product_id.into(), name_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_tags(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_tags: {:#?}", product);

    let mut product_tags_ids: Vec<u64> = Vec::new();

    if let Some(tags_ids) = &product.tags {
        product_tags_ids = tags_ids
            .iter()
            .map(|s| s.tag_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Producttags::Table)
        .cond_where(
            Cond::all()
                .add(Expr::col(Producttags::ProducttagsProductId).eq(product.product_id))
                .add(Expr::col(Producttags::ProducttagsTagId).is_not_in(product_tags_ids.clone())),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for name_id in product_tags_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Producttags::Table)
            .columns([
                Producttags::ProducttagsProductId,
                Producttags::ProducttagsTagId,
            ])
            .values([product.product_id.into(), name_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_synonyms(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_synonyms: {:#?}", product);

    let mut product_synonyms_ids: Vec<u64> = Vec::new();

    if let Some(synonyms_ids) = &product.synonyms {
        product_synonyms_ids = synonyms_ids
            .iter()
            .map(|s| s.name_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Productsynonyms::Table)
        .cond_where(
            Cond::all()
                .add(Expr::col(Productsynonyms::ProductsynonymsProductId).eq(product.product_id))
                .add(
                    Expr::col(Productsynonyms::ProductsynonymsNameId)
                        .is_not_in(product_synonyms_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for name_id in product_synonyms_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Productsynonyms::Table)
            .columns([
                Productsynonyms::ProductsynonymsProductId,
                Productsynonyms::ProductsynonymsNameId,
            ])
            .values([product.product_id.into(), name_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_supplier_refs(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_supplier_ref: {:#?}", product);

    let mut product_supplier_refs_ids: Vec<u64> = Vec::new();

    if let Some(supplier_refs_ids) = &product.supplier_refs {
        product_supplier_refs_ids = supplier_refs_ids
            .iter()
            .map(|c| c.supplier_ref_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Productsupplierrefs::Table)
        .cond_where(
            Cond::all()
                .add(
                    Expr::col(Productsupplierrefs::ProductsupplierrefsProductId)
                        .eq(product.product_id),
                )
                .add(
                    Expr::col(Productsupplierrefs::ProductsupplierrefsSupplierRefId)
                        .is_not_in(product_supplier_refs_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for class_of_compound_id in product_supplier_refs_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Productsupplierrefs::Table)
            .columns([
                Productsupplierrefs::ProductsupplierrefsProductId,
                Productsupplierrefs::ProductsupplierrefsSupplierRefId,
            ])
            .values([product.product_id.into(), class_of_compound_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_hazard_statements(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_hazard_statement: {:#?}", product);

    let mut product_hazard_statements_ids: Vec<u64> = Vec::new();

    if let Some(hazard_statements_ids) = &product.hazard_statements {
        product_hazard_statements_ids = hazard_statements_ids
            .iter()
            .map(|c| c.hazard_statement_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Producthazardstatements::Table)
        .cond_where(
            Cond::all()
                .add(
                    Expr::col(Producthazardstatements::ProducthazardstatementsProductId)
                        .eq(product.product_id),
                )
                .add(
                    Expr::col(Producthazardstatements::ProducthazardstatementsHazardStatementId)
                        .is_not_in(product_hazard_statements_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for class_of_compound_id in product_hazard_statements_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Producthazardstatements::Table)
            .columns([
                Producthazardstatements::ProducthazardstatementsProductId,
                Producthazardstatements::ProducthazardstatementsHazardStatementId,
            ])
            .values([product.product_id.into(), class_of_compound_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_precautionary_statements(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "create_update_product_precautionary_statement: {:#?}",
        product
    );

    let mut product_precautionary_statements_ids: Vec<u64> = Vec::new();

    if let Some(precautionary_statements_ids) = &product.precautionary_statements {
        product_precautionary_statements_ids = precautionary_statements_ids
            .iter()
            .map(|c| c.precautionary_statement_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Productprecautionarystatements::Table)
        .cond_where(
            Cond::all()
                .add(
                    Expr::col(Productprecautionarystatements::ProductprecautionarystatementsProductId)
                        .eq(product.product_id),
                )
                .add(
                    Expr::col(Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId)
                        .is_not_in(product_precautionary_statements_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for class_of_compound_id in product_precautionary_statements_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Productprecautionarystatements::Table)
            .columns([
                Productprecautionarystatements::ProductprecautionarystatementsProductId,
                Productprecautionarystatements::ProductprecautionarystatementsPrecautionaryStatementId,
            ])
            .values([product.product_id.into(), class_of_compound_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

fn create_update_product_classes_of_compound(
    db_transaction: &Transaction,
    product: &ProductStruct,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("create_update_product_classes_of_compound: {:#?}", product);

    let mut product_classes_of_compounds_ids: Vec<u64> = Vec::new();

    if let Some(classes_of_compounds_ids) = &product.classes_of_compound {
        product_classes_of_compounds_ids = classes_of_compounds_ids
            .iter()
            .map(|c| c.class_of_compound_id.unwrap_or_default())
            .collect();
    }

    // Delete query.
    let (sql_query, sql_values) = Query::delete()
        .from_table(Productclassesofcompounds::Table)
        .cond_where(
            Cond::all()
                .add(
                    Expr::col(Productclassesofcompounds::ProductclassesofcompoundsProductId)
                        .eq(product.product_id),
                )
                .add(
                    Expr::col(
                        Productclassesofcompounds::ProductclassesofcompoundsClassOfCompoundId,
                    )
                    .is_not_in(product_classes_of_compounds_ids.clone()),
                ),
        )
        .build_rusqlite(SqliteQueryBuilder);

    debug!("sql_query: {}", sql_query.clone().as_str());
    debug!("sql_values: {:?}", sql_values);

    _ = db_transaction.execute(sql_query.as_str(), &*sql_values.as_params());

    // Insert query.
    for class_of_compound_id in product_classes_of_compounds_ids {
        let sql_values: RusqliteValues = RusqliteValues(vec![]);
        let sql_query = Query::insert()
            .into_table(Productclassesofcompounds::Table)
            .columns([
                Productclassesofcompounds::ProductclassesofcompoundsProductId,
                Productclassesofcompounds::ProductclassesofcompoundsClassOfCompoundId,
            ])
            .values([product.product_id.into(), class_of_compound_id.into()])?
            .to_string(SqliteQueryBuilder);

        debug!("sql_query: {}", sql_query.clone().as_str());
        debug!("sql_values: {:?}", sql_values);

        _ = db_transaction.execute(&sql_query, &*sql_values.as_params())?;
    }

    Ok(())
}

pub fn delete_product(
    db_connection: &mut Connection,
    product_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("delete_product: {:#?}", product_id);

    let (delete_sql, delete_values) = Query::delete()
        .from_table(Product::Table)
        .and_where(Expr::col(Product::ProductId).eq(product_id))
        .build_rusqlite(SqliteQueryBuilder);

    _ = db_connection.execute(delete_sql.as_str(), &*delete_values.as_params())?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::{connect_test, init_db, insert_fake_values};
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = connect_test();
        init_db(&mut db_connection).unwrap();
        insert_fake_values(&mut db_connection).unwrap();
        db_connection
    }

    #[test]
    fn test_get_products() {
        init_logger();

        let db_connection = init_test_db();

        // info!("testing total result");
        // let filter = RequestFilter {
        //     order_by: Some("name".to_string()),
        //     ..Default::default()
        // };
        // let products: Vec<chimitheque_types::product::Product>;
        // let count: usize;
        // (products, count) = get_products(&db_connection, filter, 143).unwrap();
        //
        // info!("count: {}", count);
        // assert!(count > 0);

        info!("testing storage and permission join");
        let filter = RequestFilter {
            storage_to_destroy: true,
            ..Default::default()
        };
        let products: Vec<chimitheque_types::product::Product>;
        let count: usize;
        (products, count) = get_products(&db_connection, filter, 143).unwrap();

        info!("count: {}", count);
        info!("products: {:?}", products);
    }
}
