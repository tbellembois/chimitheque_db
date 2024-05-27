use chimitheque_types::signalword::Signalword;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct SignalwordWrapper(pub Signalword);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::searchable::tests::test_searchable;

    #[test]
    fn test_get_signalwords() {
        test_searchable(
            Signalword {
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
