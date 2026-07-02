#[derive(Clone, Debug)]
pub(crate) enum TacticSource {
    None,
    FmfFile,
    LiveMemory,
}

#[derive(Clone, Debug)]
pub(crate) struct TacticRecord {
    pub(crate) source: TacticSource,
    pub(crate) formation: Option<String>,
    pub(crate) roles: Vec<String>,
    pub(crate) duties: Vec<String>,
}
