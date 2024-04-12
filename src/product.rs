use sea_query::Iden;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Product {
    Table,
    Person,
    ProductId,
    ProductInchi,
    ProductInchikey,
    ProductCanonicalSmiles,
    ProductMolecularweight,
    Empiricalformula,
    Signalword,
    Name,
    Casnumber,
    Cenumber,
    UnitMolecularweight,
    ProductTwodformula,
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Producthazardstatements {
    Table,
    ProducthazardstatementsProductId,
    ProducthazardstatementsHazardstatementId,
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productprecautionarystatements {
    Table,
    ProductprecautionarystatementsProductId,
    ProductprecautionarystatementsPrecautionarystatementId,
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Productsymbols {
    Table,
    ProductsymbolsProductId,
    ProductsymbolsSymbolId,
}
