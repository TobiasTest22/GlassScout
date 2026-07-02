#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScoutKnowledgeLevel {
    FullyKnown,
    PartiallyKnown,
    Unknown,
}

#[derive(Clone, Debug)]
pub(crate) struct ScoutKnowledgeRecord {
    pub(crate) level: ScoutKnowledgeLevel,
    pub(crate) confidence: u8,
    pub(crate) last_scouted_date: Option<String>,
    pub(crate) report_reliability: Option<String>,
}
