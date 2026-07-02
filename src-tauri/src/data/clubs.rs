#[derive(Clone, Debug)]
pub(crate) struct ClubRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) nation: Option<String>,
    pub(crate) league: Option<String>,
}
