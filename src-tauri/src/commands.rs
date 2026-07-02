pub(crate) const REGISTERED_COMMANDS: &[&str] = &[
    "connector_status",
    "connector_snapshot",
    "load_active_save",
    "search_indexed_players",
    "indexed_players_by_ids",
    "mapping_lab_status",
    "mapping_lab_capture",
    "mapping_lab_compare",
    "club_logo_data",
    "player_face_data",
    "filter_observations",
];

pub(crate) fn registered_commands() -> &'static [&'static str] {
    REGISTERED_COMMANDS
}
