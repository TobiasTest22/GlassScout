#![recursion_limit = "256"]

mod connector;
mod mapping_lab;
mod player_face;
mod visibility;

use tauri_plugin_sql::{Migration, MigrationKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "initial_revealed_data_schema",
        sql: include_str!("../migrations/001_initial.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:glassscout.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            connector::connector_status,
            connector::connector_snapshot,
            connector::load_active_save,
            connector::search_indexed_players,
            connector::indexed_players_by_ids,
            mapping_lab::mapping_lab_status,
            mapping_lab::mapping_lab_capture,
            mapping_lab::mapping_lab_compare,
            player_face::club_logo_data,
            player_face::player_face_data,
            visibility::filter_observations
        ])
        .run(tauri::generate_context!())
        .expect("error while running GlassScout FM26");
}
