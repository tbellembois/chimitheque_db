use sea_query::Iden;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Permission {
    Table,
    PermissionId,
    Person,
    PermissionPermName,
    PermissionItemName,
    PermissionEntityId,
}
