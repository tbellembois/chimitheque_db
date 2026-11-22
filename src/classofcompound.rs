use chimitheque_types::classofcompound::ClassOfCompound as ClassOfCompoundStruct;
use sea_query::Iden;
use serde::Serialize;

/// Table and column names of the `class_of_compound` table, for `sea_query`.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum ClassOfCompound {
    /// The table.
    Table,
    /// The `class_of_compound_id` column.
    ClassOfCompoundId,
    /// The `class_of_compound_label` column.
    ClassOfCompoundLabel,
}

/// A class of compound read from a database row.
#[derive(Debug, Serialize, Default)]
pub struct ClassOfCompoundWrapper(pub ClassOfCompoundStruct);
