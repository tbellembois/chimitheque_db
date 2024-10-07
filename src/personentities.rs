use sea_query::Iden;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Personentities {
    Table,
    PersonentitiesPersonId,
    PersonentitiesEntityId,
}
