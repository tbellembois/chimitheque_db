use crate::{
    casnumber::Casnumber,
    category::Category,
    cenumber::Cenumber,
    classofcompound::Classofcompound,
    empiricalformula::Empiricalformula,
    hazardstatement::Hazardstatement,
    linearformula::Linearformula,
    name::Name,
    person::Person,
    physicalstate::Physicalstate,
    precautionarystatement::Precautionarystatement,
    producer::Producer,
    producerref::Producerref,
    productclassofcompound::{Productclassofcompound, ProductclassofcompoundWrapper},
    producthazardstatements::{Producthazardstatements, ProducthazardstatementsWrapper},
    productprecautionarystatements::{
        Productprecautionarystatements, ProductprecautionarystatementsWrapper,
    },
    productsupplierrefs::{Productsupplierrefs, ProductsupplierrefsWrapper},
    productsymbols::{Productsymbols, ProductsymbolsWrapper},
    productsynonyms::{Productsynonyms, ProductsynonymsWrapper},
    producttags::{Producttags, ProducttagsWrapper},
    signalword::Signalword,
    supplier::Supplier,
    supplierref::Supplierref,
    symbol::Symbol,
    tag::Tag,
    unit::Unit,
};
use chimitheque_types::{
    casnumber::Casnumber as CasnumberStruct,
    category::Category as CategoryStruct,
    cenumber::Cenumber as CenumberStruct,
    empiricalformula::Empiricalformula as EmpiricalformulaStruct,
    hazardstatement::Hazardstatement as HazardstatementStruct,
    linearformula::Linearformula as LinearformulaStruct,
    name::Name as NameStruct,
    person::Person as PersonStruct,
    physicalstate::Physicalstate as PhysicalstateStruct,
    producer::Producer as ProducerStruct,
    producerref::Producerref as ProducerrefStruct,
    product::Product as ProductStruct,
    productsupplierrefs::Productsupplierrefs as ProductsupplierrefsStruct,
    requestfilter::RequestFilter,
    signalword::Signalword as SignalwordStruct,
    tag::Tag as TagStruct,
    unit::Unit as UnitStruct,
    unittype::{ParseUnitTypeError, UnitType},
};
use log::debug;
use rusqlite::{Connection, Row};
use sea_query::{
    Alias, ColumnRef, Expr, Iden, IntoColumnRef, JoinType, Order, Query, SqliteQueryBuilder,
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
        let maybe_unit_molecularweight_type_string: Option<String> =
            row.get_unwrap("unit_molecularweight_unit_type");
        let maybe_casnumber: Option<u64> = row.get_unwrap("casnumber_id");
        let maybe_cenumber: Option<u64> = row.get_unwrap("cenumber_id");
        let maybe_empiricalformula: Option<u64> = row.get_unwrap("empiricalformula_id");
        let maybe_linearformula: Option<u64> = row.get_unwrap("linearformula_id");
        let maybe_physicalstate: Option<u64> = row.get_unwrap("physicalstate_id");
        let maybe_signalword: Option<u64> = row.get_unwrap("signalword_id");
        let maybe_category: Option<u64> = row.get_unwrap("category_id");
        let maybe_producerref: Option<u64> = row.get_unwrap("producerref_id");
        let maybe_unit_temperature: Option<u64> = row.get_unwrap("unit_temperature_unit_id");
        let maybe_unit_molecularweight: Option<u64> =
            row.get_unwrap("unit_molecularweight_unit_id");

        // Extract unit temperature type if some.
        let unit_temperature_type: UnitType;
        if maybe_unit_temperature.is_some() {
            unit_temperature_type =
                UnitType::from_str(&maybe_unit_temperature_type_string.unwrap())?;
        } else {
            unit_temperature_type = Default::default();
        }

        // Extract unit molecular weight type if some.
        let unit_molecularweight_type: UnitType;
        if maybe_unit_molecularweight.is_some() {
            unit_molecularweight_type =
                UnitType::from_str(&maybe_unit_molecularweight_type_string.unwrap())?;
        } else {
            unit_molecularweight_type = Default::default();
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
            cas_number: maybe_casnumber.map(|_| CasnumberStruct {
                casnumber_id: row.get_unwrap("casnumber_id"),
                casnumber_label: row.get_unwrap("casnumber_label"),
                ..Default::default()
            }),
            ce_number: maybe_cenumber.map(|_| CenumberStruct {
                cenumber_id: row.get_unwrap("cenumber_id"),
                cenumber_label: row.get_unwrap("cenumber_label"),
                ..Default::default()
            }),
            empirical_formula: maybe_empiricalformula.map(|_| EmpiricalformulaStruct {
                empiricalformula_id: row.get_unwrap("empiricalformula_id"),
                empiricalformula_label: row.get_unwrap("empiricalformula_label"),
                ..Default::default()
            }),
            linear_formula: maybe_linearformula.map(|_| LinearformulaStruct {
                linearformula_id: row.get_unwrap("linearformula_id"),
                linearformula_label: row.get_unwrap("linearformula_label"),
                ..Default::default()
            }),
            physical_state: maybe_physicalstate.map(|_| PhysicalstateStruct {
                physicalstate_id: row.get_unwrap("physicalstate_id"),
                physicalstate_label: row.get_unwrap("physicalstate_label"),
                ..Default::default()
            }),
            signal_word: maybe_signalword.map(|_| SignalwordStruct {
                signalword_id: row.get_unwrap("signalword_id"),
                signalword_label: row.get_unwrap("signalword_label"),
                ..Default::default()
            }),
            category: maybe_category.map(|_| CategoryStruct {
                category_id: row.get_unwrap("category_id"),
                category_label: row.get_unwrap("category_label"),
                ..Default::default()
            }),
            producer_ref: maybe_producerref.map(|_| ProducerrefStruct {
                producerref_id: row.get_unwrap("producerref_id"),
                producerref_label: row.get_unwrap("producerref_label"),
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
            unit_molecular_weight: maybe_unit_molecularweight.map(|_| UnitStruct {
                unit_id: row.get_unwrap("unit_molecularweight_unit_id"),
                unit_label: row.get_unwrap("unit_molecularweight_unit_label"),
                unit_multiplier: row.get_unwrap("unit_molecularweight_unit_multiplier"),
                unit_type: unit_molecularweight_type,
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
            ..Default::default()
        }))
    }
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
                Productclassofcompound::ProductclassofcompoundProductId,
                Productclassofcompound::ProductclassofcompoundClassofcompoundId,
            ])
            .column(Classofcompound::ClassofcompoundLabel)
            .from(Productclassofcompound::Table)
            //
            // classofcompounds
            //
            .join(
                JoinType::LeftJoin,
                Classofcompound::Table,
                Expr::col((
                    Productclassofcompound::Table,
                    Productclassofcompound::ProductclassofcompoundClassofcompoundId,
                ))
                .equals((Classofcompound::Table, Classofcompound::ClassofcompoundId)),
            )
            .and_where(
                Expr::col(Productclassofcompound::ProductclassofcompoundProductId).eq(product_id),
            )
            .build_rusqlite(SqliteQueryBuilder);

        debug!("sql: {}", sql.clone().as_str());
        debug!("values: {:?}", values);

        // Perform select query.
        let mut stmt = db_connection.prepare(sql.as_str())?;
        let rows = stmt.query_map(&*values.as_params(), |row| {
            Ok(ProductclassofcompoundWrapper::from(row))
        })?;

        // Populate product classes of compound.
        let mut classes_of_compound: Vec<chimitheque_types::classofcompound::Classofcompound> =
            vec![];
        for row in rows {
            let product_class_of_compound_wrapper = row?;
            classes_of_compound.push(chimitheque_types::classofcompound::Classofcompound {
                match_exact_search: false,
                classofcompound_id: product_class_of_compound_wrapper
                    .0
                    .productclassofcompound_classofcompound_id,
                classofcompound_label: product_class_of_compound_wrapper
                    .0
                    .productclassofcompound_classofcompound_label,
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
                Producthazardstatements::ProducthazardstatementsHazardstatementId,
            ])
            .column(Hazardstatement::HazardstatementLabel)
            .column(Hazardstatement::HazardstatementReference)
            .column(Hazardstatement::HazardstatementCmr)
            .from(Producthazardstatements::Table)
            //
            // hazardstatement
            //
            .join(
                JoinType::LeftJoin,
                Hazardstatement::Table,
                Expr::col((
                    Producthazardstatements::Table,
                    Producthazardstatements::ProducthazardstatementsHazardstatementId,
                ))
                .equals((Hazardstatement::Table, Hazardstatement::HazardstatementId)),
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
        let mut hazard_statements: Vec<chimitheque_types::hazardstatement::Hazardstatement> =
            vec![];
        for row in rows {
            let product_hazardstatements_wrapper = row?;
            hazard_statements.push(chimitheque_types::hazardstatement::Hazardstatement {
                match_exact_search: false,
                hazardstatement_id: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazardstatement_id,
                hazardstatement_label: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazardstatement_label,
                hazardstatement_reference: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazardstatement_reference,
                hazardstatement_cmr: product_hazardstatements_wrapper
                    .0
                    .producthazardstatements_hazardstatement_cmr,
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
                Productprecautionarystatements::ProductprecautionarystatementsPrecautionarystatementId,
            ])
            .column(Precautionarystatement::PrecautionarystatementLabel)
            .column(Precautionarystatement::PrecautionarystatementReference)
            .from(Productprecautionarystatements::Table)
            //
            // precautionarystatement
            //
            .join(
                JoinType::LeftJoin,
                Precautionarystatement::Table,
                Expr::col((
                    Productprecautionarystatements::Table,
                    Productprecautionarystatements::ProductprecautionarystatementsPrecautionarystatementId,
                ))
                .equals((Precautionarystatement::Table, Precautionarystatement::PrecautionarystatementId)),
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
            chimitheque_types::precautionarystatement::Precautionarystatement,
        > = vec![];
        for row in rows {
            let product_precautionarystatements_wrapper = row?;
            precautionary_statements.push(
                chimitheque_types::precautionarystatement::Precautionarystatement {
                    match_exact_search: false,
                    precautionarystatement_id: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionarystatement_id,
                    precautionarystatement_label: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionarystatement_label,
                    precautionarystatement_reference: product_precautionarystatements_wrapper
                        .0
                        .productprecautionarystatements_precautionarystatement_reference,
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
                Productsupplierrefs::ProductsupplierrefsSupplierrefId,
            ])
            .columns([Supplierref::SupplierrefId, Supplierref::SupplierrefLabel])
            .columns([Supplier::SupplierId, Supplier::SupplierLabel])
            .from(Productsupplierrefs::Table)
            //
            // supplierref
            //
            .join(
                JoinType::LeftJoin,
                Supplierref::Table,
                Expr::col((
                    Productsupplierrefs::Table,
                    Productsupplierrefs::ProductsupplierrefsSupplierrefId,
                ))
                .equals((Supplierref::Table, Supplierref::SupplierrefId)),
            )
            //
            // supplier
            //
            .join(
                JoinType::LeftJoin,
                Supplier::Table,
                Expr::col((Supplierref::Table, Supplierref::Supplier))
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
        let mut supplier_refs: Vec<chimitheque_types::supplierref::Supplierref> = vec![];
        for row in rows {
            let product_supplierrefs_wrapper = row?;
            supplier_refs.push(chimitheque_types::supplierref::Supplierref {
                match_exact_search: false,
                supplierref_id: product_supplierrefs_wrapper
                    .0
                    .productsupplierrefs_supplierref_id,
                supplierref_label: product_supplierrefs_wrapper
                    .0
                    .productsupplierrefs_supplierref_label,
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
            "cas_number" => (Casnumber::Table, Casnumber::CasnumberLabel).into_column_ref(),
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
            Casnumber::Table,
            Expr::col((Product::Table, Product::CasNumber))
                .equals((Casnumber::Table, Casnumber::CasnumberId)),
        )
        //
        // ce number
        //
        .join(
            JoinType::LeftJoin,
            Cenumber::Table,
            Expr::col((Product::Table, Product::CeNumber))
                .equals((Cenumber::Table, Cenumber::CenumberId)),
        )
        //
        // empirical formula
        //
        .join(
            JoinType::LeftJoin,
            Empiricalformula::Table,
            Expr::col((Product::Table, Product::EmpiricalFormula)).equals((
                Empiricalformula::Table,
                Empiricalformula::EmpiricalformulaId,
            )),
        )
        //
        // linear formula
        //
        .join(
            JoinType::LeftJoin,
            Linearformula::Table,
            Expr::col((Product::Table, Product::LinearFormula))
                .equals((Linearformula::Table, Linearformula::LinearformulaId)),
        )
        //
        // physical state
        //
        .join(
            JoinType::LeftJoin,
            Physicalstate::Table,
            Expr::col((Product::Table, Product::PhysicalState))
                .equals((Physicalstate::Table, Physicalstate::PhysicalstateId)),
        )
        //
        // signal word
        //
        .join(
            JoinType::LeftJoin,
            Signalword::Table,
            Expr::col((Product::Table, Product::SignalWord))
                .equals((Signalword::Table, Signalword::SignalwordId)),
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
            Producerref::Table,
            Expr::col((Product::Table, Product::ProducerRef))
                .equals((Producerref::Table, Producerref::ProducerrefId)),
        )
        .join(
            JoinType::LeftJoin,
            Producer::Table,
            Expr::col((Producerref::Table, Producerref::Producer))
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
            Alias::new("unit_molecularweight"),
            Expr::col((Product::Table, Product::UnitMolecularWeight))
                .equals((Alias::new("unit_molecularweight"), Unit::UnitId)),
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
        //
        // filters
        //
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
            filter.name.is_some(),
            |q| {
                q.and_where(Expr::col(Product::Name).eq(filter.name.unwrap()));
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
        .expr(Expr::col((Casnumber::Table, Casnumber::CasnumberId)))
        .expr(Expr::col((Casnumber::Table, Casnumber::CasnumberLabel)))
        .expr(Expr::col((Cenumber::Table, Cenumber::CenumberId)))
        .expr(Expr::col((Cenumber::Table, Cenumber::CenumberLabel)))
        .expr(Expr::col((
            Empiricalformula::Table,
            Empiricalformula::EmpiricalformulaId,
        )))
        .expr(Expr::col((
            Empiricalformula::Table,
            Empiricalformula::EmpiricalformulaLabel,
        )))
        .expr(Expr::col((
            Linearformula::Table,
            Linearformula::LinearformulaId,
        )))
        .expr(Expr::col((
            Linearformula::Table,
            Linearformula::LinearformulaLabel,
        )))
        .expr(Expr::col((
            Physicalstate::Table,
            Physicalstate::PhysicalstateId,
        )))
        .expr(Expr::col((
            Physicalstate::Table,
            Physicalstate::PhysicalstateLabel,
        )))
        .expr(Expr::col((Category::Table, Category::CategoryId)))
        .expr(Expr::col((Category::Table, Category::CategoryLabel)))
        .expr(Expr::col((Signalword::Table, Signalword::SignalwordId)))
        .expr(Expr::col((Signalword::Table, Signalword::SignalwordLabel)))
        .expr(Expr::col((Producerref::Table, Producerref::ProducerrefId)))
        .expr(Expr::col((
            Producerref::Table,
            Producerref::ProducerrefLabel,
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
            Expr::col((Alias::new("unit_molecularweight"), Alias::new("unit_id"))),
            Alias::new("unit_molecularweight_unit_id"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_molecularweight"), Alias::new("unit_label"))),
            Alias::new("unit_molecularweight_unit_label"),
        )
        .expr_as(
            Expr::col((
                Alias::new("unit_molecularweight"),
                Alias::new("unit_multiplier"),
            )),
            Alias::new("unit_molecularweight_unit_multiplier"),
        )
        .expr_as(
            Expr::col((Alias::new("unit_molecularweight"), Alias::new("unit_type"))),
            Alias::new("unit_molecularweight_unit_type"),
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
        init_db(&mut db_connection).unwrap();
        insert_fake_values(&mut db_connection).unwrap();

        info!("testing total result");
        let filter = RequestFilter {
            order_by: Some("name".to_string()),
            ..Default::default()
        };
        let products: Vec<chimitheque_types::product::Product>;
        let count: usize;
        (products, count) = get_products(&db_connection, filter, 2).unwrap();

        assert!(count > 0);

        for product in products.iter() {
            if product.product_id.eq(&5973) {
                info!("product:{:?}", product);
                assert!(product.empirical_formula.is_some())
            }
        }
    }
}
