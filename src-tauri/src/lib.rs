mod connector;
mod tactic_file;
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
        .plugin(tauri_plugin_dialog::init())
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
            tactic_file::import_tactic_file,
            tactic_file::tactic_file_status,
            visibility::filter_observations
        ])
        .run(tauri::generate_context!())
        .expect("error while running GlassScout FM26");
}
