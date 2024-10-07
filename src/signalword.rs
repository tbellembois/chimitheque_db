use chimitheque_types::signalword::SignalWord as SignalWordStruct;
use sea_query::Iden;
use serde::Serialize;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
pub enum SignalWord {
    Table,
    SignalWordId,
    SignalWordLabel,
}

#[derive(Debug, Serialize, Default)]
pub struct SignalwordWrapper(pub SignalWordStruct);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_signal_words() {
        test_searchable(
            SignalWordStruct {
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
