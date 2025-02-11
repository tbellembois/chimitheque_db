use sea_query::Iden;

// Entity managers.
#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Personentities {
    Table,
    PersonentitiesPersonId,
    PersonentitiesEntityId,
}
