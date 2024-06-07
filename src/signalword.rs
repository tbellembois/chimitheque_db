use chimitheque_types::signalword::Signalword as SignalwordStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum Signalword {
    Table,
    SignalwordId,
    SignalwordLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct SignalwordWrapper(pub SignalwordStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_signalwords() {
        test_searchable(
            SignalwordStruct {
                ..Default::default()
            },
            vec![
                "signalword1",
                "aa signalword1",
                "bb sIgNaLwOrD1",
                "signalword2",
                "signalword3",
            ],
            3,
            "signalword1",
        )
    }
}
