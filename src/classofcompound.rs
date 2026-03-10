use chimitheque_types::classofcompound::ClassOfCompound as ClassOfCompoundStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum ClassOfCompound {
    Table,
    ClassOfCompoundId,
    ClassOfCompoundLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct ClassOfCompoundWrapper(pub ClassOfCompoundStruct);
