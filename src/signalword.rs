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
