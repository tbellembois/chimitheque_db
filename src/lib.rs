//! Database layer of Chimithèque, a management application for chemical
//! products, biological reagents and laboratory consumables.
//!
//! It reads and writes the `SQLite` database. The data is validated by
//! `chimitheque_types` before being saved here: the database schema is the
//! last safety net.
#![warn(missing_docs)]

/// Product bookmarks of a person.
pub mod bookmark;
/// Storage borrowings.
pub mod borrowing;
/// Functions used by the casbin permission matcher.
pub mod casbin;
/// CAS numbers.
pub mod casnumber;
/// Product categories.
pub mod category;
/// CE numbers.
pub mod cenumber;
/// Classes of compounds.
pub mod classofcompound;
/// Empirical formulas.
pub mod empiricalformula;
/// Entities and their managers.
pub mod entity;
/// Entity managers table.
pub mod entitypeople;
/// GHS hazard statements.
pub mod hazardstatement;
/// Database connection, creation and base data.
pub mod init;
/// Linear formulas.
pub mod linearformula;
/// Product names.
pub mod name;
/// Permissions of people.
pub mod permission;
/// People, their entities, managed entities and permissions.
pub mod person;
/// Entity members table.
pub mod personentities;
/// Physical states.
pub mod physicalstate;
/// GHS precautionary statements.
pub mod precautionarystatement;
/// Producers.
pub mod producer;
/// Producer references.
pub mod producerref;
/// Products.
pub mod product;
/// Product classes of compounds table.
pub mod productclassesofcompounds;
/// Product hazard statements table.
pub mod producthazardstatements;
/// Product precautionary statements table.
pub mod productprecautionarystatements;
/// Product supplier references table.
pub mod productsupplierrefs;
/// Product symbols table.
pub mod productsymbols;
/// Product synonyms table.
pub mod productsynonyms;
/// Product tags table.
pub mod producttags;
pub mod pubchemproduct;
pub mod reference;
/// Generic functions for the types with only an id and a text.
pub mod searchable;
/// GHS signal words.
pub mod signalword;
/// Product stocks.
pub mod stock;
/// Storages.
pub mod storage;
/// Store locations.
pub mod storelocation;
/// Suppliers.
pub mod supplier;
/// Supplier references.
pub mod supplierref;
/// GHS symbols.
pub mod symbol;
/// Product tags.
pub mod tag;
/// Test helpers.
pub mod test_utils;
/// Units.
pub mod unit;
