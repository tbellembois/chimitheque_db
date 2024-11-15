use crate::{
    bookmark::Bookmark,
    borrowing::Borrowing,
    casnumber::CasNumber,
    category::Category,
    cenumber::CeNumber,
    classofcompound::ClassOfCompound,
    empiricalformula::EmpiricalFormula,
    entity::Entity,
    hazardstatement::HazardStatement,
    linearformula::LinearFormula,
    name::Name,
    permission::Permission,
    person::Person,
    physicalstate::PhysicalState,
    precautionarystatement::PrecautionaryStatement,
    producer::Producer,
    producerref::ProducerRef,
    productclassesofcompounds::{Productclassesofcompounds, ProductclassesofcompoundsWrapper},
    producthazardstatements::{Producthazardstatements, ProducthazardstatementsWrapper},
    productprecautionarystatements::{
        Productprecautionarystatements, ProductprecautionarystatementsWrapper,
    },
    productsupplierrefs::{Productsupplierrefs, ProductsupplierrefsWrapper},
    productsymbols::{Productsymbols, ProductsymbolsWrapper},
    productsynonyms::{Productsynonyms, ProductsynonymsWrapper},
    producttags::{Producttags, ProducttagsWrapper},
    signalword::SignalWord,
    storage::Storage,
    storelocation::StoreLocation,
    supplier::Supplier,
    supplierref::SupplierRef,
    symbol::Symbol,
    tag::Tag,
    unit::Unit,
};
use chimitheque_types::{
    casnumber::CasNumber as CasNumberStruct,
    category::Category as CategoryStruct,
    cenumber::CeNumber as CeNumberStruct,
    empiricalformula::EmpiricalFormula as EmpiricalFormulaStruct,
    linearformula::LinearFormula as LinearFormulaStruct,
    name::Name as NameStruct,
    person::Person as PersonStruct,
    physicalstate::PhysicalState as PhysicalStateStruct,
    producer::Producer as ProducerStruct,
    producerref::ProducerRef as ProducerRefStruct,
    product::Product as ProductStruct,
    requestfilter::RequestFilter,
    signalword::SignalWord as SignalWordStruct,
    unit::Unit as UnitStruct,
    unittype::{ParseUnitTypeError, UnitType},
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    any, Alias, ColumnRef, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
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
    type Error = ParseUnitTypeError;

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

        // Extract unit temperature type if some.
        let unit_temperature_type: UnitType;
        if maybe_unit_temperature.is_some() {
            unit_temperature_type =
                UnitType::from_str(&maybe_unit_temperature_type_string.unwrap())?;
        } else {
            unit_temperature_type = Default::default();
        }

        // Extract unit molecular weight type if some.
        let unit_molecular_weight_type: UnitType;
        if maybe_unit_molecular_weight.is_some() {
            unit_molecular_weight_type =
                UnitType::from_str(&maybe_unit_molecular_weight_type_string.unwrap())?;
        } else {
            unit_molecular_weight_type = Default::default();
        }

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
            product_molecula_weight: row.get_unwrap("product_molecular_weight"),
            product_temperature: row.get_unwrap("product_temperature"),
            product_sheet: row.get_unwrap("product_sheet"),
            product_number_per_carton: row.get_unwrap("product_number_per_carton"),
            product_number_per_bag: row.get_unwrap("product_number_per_bag"),
            product_sl: row.get_unwrap("product_sl"),
            product_hs_cmr: row.get_unwrap("product_hs_cmr"),
            ..Default::default()
        }))
    }
}

fn populate_product_sc(
    db_connection: &Connection,
    products: &mut Vec<ProductStruct>,
    person_id: u64,
    total: bool,
    archived: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("person_id:{:?}", person_id);
    debug!("archived:{:?}", archived);

    for product in products.iter_mut() {
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
                                Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
                                    .is_in(["all", "storages"]),
                            )
                            .and(
                                Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
                                    .is_in(["r", "w", "all"]),
                            )
                            .and(
                                Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                                    .equals(Entity::EntityId)
                                    .or(Expr::col(Entity::EntityId).is_null()) // products with no storages for non admins
                                    .or(Expr::col((
                                        Alias::new("perm"),
                                        Alias::new("permission_entity_id"),
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
                |_| {},
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                name_id: product_synonym_wrapper.0.productsynonyms_name_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                class_of_compound_id: product_class_of_compound_wrapper
                    .0
                    .productclassesofcompounds_class_of_compound_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                symbol_id: product_symbols_wrapper.0.productsymbols_symbol_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                hazard_statement_id: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazard_statement_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                    precautionary_statement_id: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionary_statement_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                supplier_ref_id: product_supplierrefs_wrapper
                    .0
                    .productsupplierrefs_supplier_ref_id,
                supplier_ref_label: product_supplierrefs_wrapper
                    .0
                    .productsupplierrefs_supplier_ref_label,
                supplier: chimitheque_types::supplier::Supplier {
                    match_exact_search: false,
                    supplier_id: product_supplierrefs_wrapper
                        .0
                        .productsupplierrefs_supplier_id,
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
    products: &mut Vec<ProductStruct>,
) -> Result<(), Box<dyn std::error::Error>> {
    for product in products.iter_mut() {
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
                tag_id: product_tags_wrapper.0.producttags_tag_id,
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
                    Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
                        .is_in(["all", "storages"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
                        .is_in(["r", "w", "all"]),
                )
                .and(
                    Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                        .equals(Entity::EntityId)
                        .or( Expr::col(Entity::EntityId).is_null()) // products with no storages for non admins
                        .or(
                            Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
                                .eq(-1),
                        ),
                ),
        )

        // .join(
        //     // storelocation
        //     JoinType::Join,
        //     StoreLocation::Table,
        //     Expr::col((Storage::Table, Storage::StoreLocation))
        //         .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
        // )
        // .join(
        //     // entity
        //     JoinType::Join,
        //     Entity::Table,
        //     Expr::col((StoreLocation::Table, StoreLocation::Entity))
        //         .equals((Entity::Table, Entity::EntityId)),
        // )
        // .join_as(
        //     JoinType::InnerJoin,
        //     Permission::Table,
        //     Alias::new("perm"),
        //     Expr::col((Alias::new("perm"), Alias::new("person")))
        //         .eq(person_id)
        //         .and(
        //             Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
        //                 .is_in(["all", "storages"]),
        //         )
        //         .and(
        //             Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
        //                 .is_in(["r", "w", "all"]),
        //         )
        //         .and(
        //             Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
        //                 .equals(Entity::EntityId)
        //                 .or(
        //                     Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
        //                         .eq(-1),
        //                 ),
        //         ),
        // )
        // .conditions(
        //     filter.entity.is_some()
        //         || filter.store_location.is_some()
        //         || filter.storage_barecode.is_some()
        //         || filter.storage_to_destroy,
        //     |q| {
        //         q.join(
        //             // storelocation
        //             JoinType::Join,
        //             StoreLocation::Table,
        //             Expr::col((Storage::Table, Storage::StoreLocation))
        //                 .equals((StoreLocation::Table, StoreLocation::StoreLocationId)),
        //         );
        //         q.join(
        //             // entity
        //             JoinType::Join,
        //             Entity::Table,
        //             Expr::col((StoreLocation::Table, StoreLocation::Entity))
        //                 .equals((Entity::Table, Entity::EntityId)),
        //         );
        //         q.join_as(
        //             // permission
        //             JoinType::InnerJoin,
        //             Permission::Table,
        //             Alias::new("perm"),
        //             Expr::col((Alias::new("perm"), Alias::new("person")))
        //                 .eq(person_id)
        //                 .and(
        //                     Expr::col((Alias::new("perm"), Alias::new("permission_item_name")))
        //                         .is_in(["all", "storages"]),
        //                 )
        //                 .and(
        //                     Expr::col((Alias::new("perm"), Alias::new("permission_perm_name")))
        //                         .is_in(["r", "w", "all"]),
        //                 )
        //                 .and(
        //                     Expr::col((Alias::new("perm"), Alias::new("permission_entity_id")))
        //                         .equals(Entity::EntityId)
        //                         .or(Expr::col((
        //                             Alias::new("perm"),
        //                             Alias::new("permission_entity_id"),
        //                         ))
        //                         .eq(-1)),
        //                 ),
        //         );
        //     },
        //     |_| {},
        // )
        //
        // filters
        //
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
            filter.product.is_some(),
            |q| {
                q.and_where(
                    Expr::col((Product::Table, Product::ProductId))
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
        .expr(Expr::col((Product::Table, Product::ProductId)).count_distinct())
        .build_rusqlite(SqliteQueryBuilder);

    debug!("count_sql: {}", count_sql.clone().as_str());
    debug!("count_values: {:?}", count_values);

    // Create select query.
    let (select_sql, select_values) = expression
        .columns([
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
        // storage count
        // .expr_as(
        //     Expr::expr(Expr::case(
        //         Expr::col((Storage::Table, Storage::StorageArchive)).eq(false),
        //         1,
        //     ))
        //     .count(),
        //     Alias::new("count_storage_not_archive"),
        // )
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

    populate_product_sc(db_connection, &mut products, person_id, false, false)?;
    populate_product_sc(db_connection, &mut products, person_id, true, false)?;
    populate_product_sc(db_connection, &mut products, person_id, false, true)?;

    debug!("products: {:#?}", products);

    Ok((products, count))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::init::{init_db, insert_fake_values};
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn init_test_db() -> Connection {
        let mut db_connection = Connection::open_in_memory().unwrap();
        init_db(&mut db_connection).unwrap();

        db_connection
    }

    #[test]
    fn test_get_products() {
        init_logger();

        let mut db_connection = init_test_db();
        insert_fake_values(&mut db_connection).unwrap();

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
    }
}
