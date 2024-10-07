use sea_query::Iden;

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
