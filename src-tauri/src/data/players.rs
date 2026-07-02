use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct IndexedPlayerRecord {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) positions: Vec<String>,
    pub(crate) managed_squad: bool,
    pub(crate) visibility_safe: bool,
    pub(crate) raw_player_address: u64,
    pub(crate) person_address: u64,
    pub(crate) contract_address: Option<u64>,
}

#[derive(Default)]
pub(crate) struct PlayerDatabaseIndex {
    pub(crate) process_id: u32,
    pub(crate) save_pointer: u64,
    pub(crate) records: HashMap<String, IndexedPlayerRecord>,
}
